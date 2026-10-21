//! CPU reference solver for SOS.
//!
//! This backend mirrors the lightweight collocated-grid scheme used by the Metal
//! backend. It is useful for CPU-only execution and reproducibility tests; it is not
//! a substitute for the missing ROMS architecture (C-grid/free surface/turbulence).
use physics::equation_of_state::density_profile;
use s_field::OceanField;

const OMEGA: f64 = 7.292_115_0e-5;
const GRAVITY: f32 = 9.80665;

pub struct CpuOceanSolver {
    land_mask: Vec<f32>,
    coriolis_by_y: Vec<f32>,
    east_u: Vec<f32>,
    east_v: Vec<f32>,
    east_w: Vec<f32>,
    nu_h: f32,
    nu_v: f32,
    bottom_drag: f32,
    pressure_iterations: usize,
    use_source_boundary: bool,
}

impl CpuOceanSolver {
    pub fn new(
        initial: &OceanField,
        horizontal_viscosity_m2_s: f32,
        vertical_viscosity_m2_s: f32,
        bottom_drag_coefficient: f32,
        use_source_boundary: bool,
    ) -> Self {
        let velocity = initial.velocity();
        let [nz, ny, nx] = velocity.shape();
        let temp = initial.temperature().field().as_slice();
        let salt = initial.salinity().field().as_slice();
        let land_mask = temp.iter().zip(salt).map(|(t,s)| if *t == 0.0 && *s == 0.0 {1.0} else {0.0}).collect::<Vec<_>>();
        let coriolis_by_y = initial.coordinates().latitude().iter().map(|lat| {
            (2.0 * OMEGA * lat.to_radians().sin()) as f32
        }).collect::<Vec<_>>();
        let plane=nx*ny;
        let u=velocity.u().time_slice(0).unwrap_or(&[]);
        let v=velocity.v().time_slice(0).unwrap_or(&[]);
        let w=velocity.w().time_slice(0).unwrap_or(&[]);
        let mut east_u=Vec::with_capacity(ny*nz);let mut east_v=Vec::with_capacity(ny*nz);let mut east_w=Vec::with_capacity(ny*nz);
        for z in 0..nz {for y in 0..ny {let i=z*plane+y*nx+nx.saturating_sub(1);east_u.push(*u.get(i).unwrap_or(&0.0));east_v.push(*v.get(i).unwrap_or(&0.0));east_w.push(*w.get(i).unwrap_or(&0.0));}}
        Self { land_mask, coriolis_by_y, east_u, east_v, east_w,
            nu_h: horizontal_viscosity_m2_s.max(0.0), nu_v: vertical_viscosity_m2_s.max(0.0),
            bottom_drag: bottom_drag_coefficient.clamp(0.0,0.1), pressure_iterations: 80,
            use_source_boundary }
    }

    pub fn step(&self, field: &mut OceanField, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 { return; }
        let shape=field.velocity().shape();let (nz,ny,nx)=(shape[0],shape[1],shape[2]);
        let n=nx*ny*nz;let plane=nx*ny;
        if n==0 {return;}
        let spacing=field.velocity().spacing();let dx=spacing[3].abs().max(1.0e-6);let dy=spacing[2].abs().max(1.0e-6);let dz=spacing[1].abs().max(1.0e-6);
        let mut u=field.velocity().u().time_slice(0).unwrap_or(&[]).to_vec();
        let mut v=field.velocity().v().time_slice(0).unwrap_or(&[]).to_vec();
        let mut w=field.velocity().w().time_slice(0).unwrap_or(&[]).to_vec();
        if u.len()!=n||v.len()!=n||w.len()!=n{return;}
        let t=field.temperature().field().as_slice().to_vec();
        let s=field.salinity().field().as_slice().to_vec();
        let (rho,rho_ref)=density_profile(&t,&s,nx,ny,nz,dz);

        // SSP-RK2 for momentum advection.
        let (ru,rv,rw)=advection_rhs(&u,&v,&w,nx,ny,nz,dx,dy,dz,&self.land_mask);
        let mut su=vec![0.0;n];let mut sv=vec![0.0;n];let mut sw=vec![0.0;n];
        for i in 0..n {su[i]=u[i]+dt*ru[i];sv[i]=v[i]+dt*rv[i];sw[i]=w[i]+dt*rw[i];}
        enforce_mask(&mut su,&mut sv,&mut sw,nx,ny,nz,&self.land_mask);
        let (r2u,r2v,r2w)=advection_rhs(&su,&sv,&sw,nx,ny,nz,dx,dy,dz,&self.land_mask);
        for i in 0..n {u[i]=0.5*(u[i]+su[i]+dt*r2u[i]);v[i]=0.5*(v[i]+sv[i]+dt*r2v[i]);w[i]=0.5*(w[i]+sw[i]+dt*r2w[i]);}

        // Exact Coriolis rotation, anisotropic background diffusion, and bottom stress.
        for z in 0..nz {for y in 0..ny {let f=*self.coriolis_by_y.get(y).unwrap_or(&0.0);let angle=f*dt;let c=angle.cos();let sn=angle.sin();for x in 0..nx {
            let i=z*plane+y*nx+x;if self.land_mask.get(i).copied().unwrap_or(0.0)>0.5 {u[i]=0.0;v[i]=0.0;w[i]=0.0;continue;}
            let old_u=u[i];let old_v=v[i];u[i]=c*old_u+sn*old_v;v[i]=c*old_v-sn*old_u;
        }}}
        let old_u=u.clone();let old_v=v.clone();let old_w=w.clone();
        for z in 1..nz.saturating_sub(1) {for y in 1..ny.saturating_sub(1) {for x in 1..nx.saturating_sub(1) {
            let i=z*plane+y*nx+x;if self.land_mask[i]>0.5 {continue;}
            let lap_h= (old_u[i+1]-2.0*old_u[i]+old_u[i-1])/(dx*dx)+(old_u[i+nx]-2.0*old_u[i]+old_u[i-nx])/(dy*dy);
            let lap_v= (old_u[i+plane]-2.0*old_u[i]+old_u[i-plane])/(dz*dz);u[i]+=dt*(self.nu_h*lap_h+self.nu_v*lap_v);
            let lap_h= (old_v[i+1]-2.0*old_v[i]+old_v[i-1])/(dx*dx)+(old_v[i+nx]-2.0*old_v[i]+old_v[i-nx])/(dy*dy);
            let lap_v= (old_v[i+plane]-2.0*old_v[i]+old_v[i-plane])/(dz*dz);v[i]+=dt*(self.nu_h*lap_h+self.nu_v*lap_v);
            let lap_h= (old_w[i+1]-2.0*old_w[i]+old_w[i-1])/(dx*dx)+(old_w[i+nx]-2.0*old_w[i]+old_w[i-nx])/(dy*dy);
            let lap_v= (old_w[i+plane]-2.0*old_w[i]+old_w[i-plane])/(dz*dz);w[i]+=dt*(self.nu_h*lap_h+self.nu_v*lap_v);
        }}}
        for z in 0..nz {for y in 0..ny {for x in 0..nx {let i=z*plane+y*nx+x;if self.land_mask[i]>0.5{continue;}
            let bottom=z+1>=nz || self.land_mask.get(i+plane).copied().unwrap_or(1.0)>0.5;
            if bottom {let speed=(u[i]*u[i]+v[i]*v[i]).sqrt();let scale=1.0/(1.0+self.bottom_drag*speed*dt/dz);u[i]*=scale;v[i]*=scale;}
            let ref_rho=rho_ref.get(z).copied().unwrap_or(1025.0).max(900.0);w[i]+=-GRAVITY*(rho[i]-ref_rho)/ref_rho*dt;
        }}}
        if self.use_source_boundary {for z in 0..nz {for y in 0..ny {let i=z*plane+y*nx+nx-1;let f=z*ny+y;u[i]=self.east_u[f];v[i]=self.east_v[f];w[i]=self.east_w[f];}}}
        enforce_mask(&mut u,&mut v,&mut w,nx,ny,nz,&self.land_mask);

        // Variable-density pressure projection with a wet-cell-aware Jacobi solve.
        let div=divergence(&u,&v,&w,nx,ny,nz,dx,dy,dz,&self.land_mask);
        let gauge=self.land_mask.iter().position(|m|*m<=0.5).unwrap_or(0);
        let mut p=vec![0.0;n];let mut next=vec![0.0;n];
        let idx=|x:usize,y:usize,z:usize|z*plane+y*nx+x;
        for _ in 0..self.pressure_iterations {for z in 0..nz {for y in 0..ny {for x in 0..nx {let i=idx(x,y,z);if i==gauge||self.land_mask[i]>0.5{next[i]=0.0;continue;}
            let ri=rho[i].max(900.0);let mut weighted=0.0;let mut sum=0.0;
            for (valid,j,invh2) in [
                (x>0,if x>0{idx(x-1,y,z)}else{i},1.0/(dx*dx)),
                (x+1<nx,if x+1<nx{idx(x+1,y,z)}else{i},1.0/(dx*dx)),
                (y>0,if y>0{idx(x,y-1,z)}else{i},1.0/(dy*dy)),
                (y+1<ny,if y+1<ny{idx(x,y+1,z)}else{i},1.0/(dy*dy)),
                (z>0,if z>0{idx(x,y,z-1)}else{i},1.0/(dz*dz)),
                (z+1<nz,if z+1<nz{idx(x,y,z+1)}else{i},1.0/(dz*dz)),
            ] {if valid&&self.land_mask[j]<=0.5 {let a=2.0/(ri+rho[j].max(900.0))*invh2;weighted+=a*p[j];sum+=a;}}
            next[i]=if sum>0.0{(weighted-div[i]/dt)/sum}else{0.0};
        }}}std::mem::swap(&mut p,&mut next);}
        for z in 0..nz {for y in 0..ny {for x in 0..nx {let i=idx(x,y,z);if self.land_mask[i]>0.5{u[i]=0.0;v[i]=0.0;w[i]=0.0;continue;}
            let ri=rho[i].max(900.0);
            if x+1<nx {let j=idx(x+1,y,z);if self.land_mask[j]<=0.5{u[i]-=dt*2.0/(ri+rho[j].max(900.0))*(p[j]-p[i])/dx;}}
            if y+1<ny {let j=idx(x,y+1,z);if self.land_mask[j]<=0.5{v[i]-=dt*2.0/(ri+rho[j].max(900.0))*(p[j]-p[i])/dy;}}
            if z+1<nz {let j=idx(x,y,z+1);if self.land_mask[j]<=0.5{w[i]+=dt*2.0/(ri+rho[j].max(900.0))*(p[j]-p[i])/dz;}}
        }}}
        if self.use_source_boundary {for z in 0..nz {for y in 0..ny {let i=z*plane+y*nx+nx-1;let f=z*ny+y;u[i]=self.east_u[f];v[i]=self.east_v[f];w[i]=self.east_w[f];}}}
        enforce_mask(&mut u,&mut v,&mut w,nx,ny,nz,&self.land_mask);
        field.velocity_mut().u_mut().replace(u);field.velocity_mut().v_mut().replace(v);field.velocity_mut().w_mut().replace(w);
    }
}

fn enforce_mask(u:&mut[f32],v:&mut[f32],w:&mut[f32],nx:usize,ny:usize,nz:usize,mask:&[f32]){
    let plane=nx*ny;for z in 0..nz{for y in 0..ny{for x in 0..nx{let i=z*plane+y*nx+x;if mask.get(i).copied().unwrap_or(1.0)>0.5{u[i]=0.0;v[i]=0.0;w[i]=0.0;}else if z==0||z+1==nz||mask.get(i+plane).copied().unwrap_or(1.0)>0.5{w[i]=0.0;}}}}
}
fn advection_rhs(u:&[f32],v:&[f32],w:&[f32],nx:usize,ny:usize,nz:usize,dx:f32,dy:f32,dz:f32,mask:&[f32])->(Vec<f32>,Vec<f32>,Vec<f32>){
    let n=u.len();let plane=nx*ny;let mut ru=vec![0.0;n];let mut rv=vec![0.0;n];let mut rw=vec![0.0;n];
    if nx<3||ny<3||nz<3{return(ru,rv,rw)}
    for z in 1..nz-1{for y in 1..ny-1{for x in 1..nx-1{let i=z*plane+y*nx+x;if mask[i]>0.5{continue;}
        let xp=i+1;let xm=i-1;let yp=i+nx;let ym=i-nx;let zp=i+plane;let zm=i-plane;
        let adv=|q:&[f32]|u[i]*(q[xp]-q[xm])/(2.0*dx)+v[i]*(q[yp]-q[ym])/(2.0*dy)-w[i]*(q[zp]-q[zm])/(2.0*dz);
        ru[i]=-adv(u);rv[i]=-adv(v);rw[i]=-adv(w);
    }}}(ru,rv,rw)
}
fn divergence(u:&[f32],v:&[f32],w:&[f32],nx:usize,ny:usize,nz:usize,dx:f32,dy:f32,dz:f32,mask:&[f32])->Vec<f32>{
    let plane=nx*ny;let mut d=vec![0.0;u.len()];if nx<3||ny<3||nz<3{return d}
    for z in 1..nz-1{for y in 1..ny-1{for x in 1..nx-1{let i=z*plane+y*nx+x;if mask[i]>0.5{continue;}d[i]=(u[i]-u[i-1])/dx+(v[i]-v[i-nx])/dy-(w[i]-w[i-plane])/dz;}}}d
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_coriolis_rotation_preserves_horizontal_speed() {
        let u=1.25_f32;let v=-0.7_f32;let f=8.0e-5_f32;let dt=120.0_f32;let a=f*dt;let c=a.cos();let s=a.sin();
        let un=c*u+s*v;let vn=c*v-s*u;
        assert!(((un*un+vn*vn)-(u*u+v*v)).abs()<1.0e-5);
    }

    #[test]
    fn constant_velocity_has_zero_advection_rhs() {
        let (nx,ny,nz)=(5,5,5);let n=nx*ny*nz;
        let u=vec![0.3_f32;n];let v=vec![-0.2_f32;n];let w=vec![0.01_f32;n];let mask=vec![0.0_f32;n];
        let (ru,rv,rw)=advection_rhs(&u,&v,&w,nx,ny,nz,1000.0,1000.0,100.0,&mask);
        assert!(ru.iter().chain(&rv).chain(&rw).all(|value|value.abs()<1.0e-7));
    }

    #[test]
    fn mask_and_rigid_lid_zero_land_and_normal_velocity() {
        let (nx,ny,nz)=(3,3,3);let n=nx*ny*nz;let mut u=vec![1.0;n];let mut v=vec![1.0;n];let mut w=vec![1.0;n];
        let mut mask=vec![0.0;n];mask[13]=1.0;
        enforce_mask(&mut u,&mut v,&mut w,nx,ny,nz,&mask);
        assert_eq!((u[13],v[13],w[13]),(0.0,0.0,0.0));
        assert_eq!(w[0],0.0);assert_eq!(w[26],0.0);
    }
}
