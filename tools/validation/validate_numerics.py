#!/usr/bin/env python3
"""Portable numerical verification harness for SOS stencil schemes.

This is an independent Python reference test, not an execution of Rust/Metal.
It intentionally reports the distinction in its output and validates the discrete
operators that SOS should match before backend equivalence can be claimed.
Requires: Python 3.10+, NumPy.
"""
from __future__ import annotations
import json, math, platform, sys
from dataclasses import asdict, dataclass
import numpy as np

@dataclass
class Result:
    name: str
    value: float
    threshold: float
    passed: bool
    detail: str

def poisson_jacobi_test(n: int = 24, iterations: int = 300) -> Result:
    # Periodic manufactured problem: p=sin(x)sin(y)sin(z), -lap(p)=3p.
    # Jacobi is deliberately tested as a solver, not assumed converged by count.
    x = 2*np.pi*np.arange(n)/n
    p_exact = (np.sin(x)[:,None,None] * np.sin(x)[None,:,None] * np.sin(x)[None,None,:]).astype(np.float64)
    h = 2*np.pi/n
    # Discrete eigenvalue for -Laplacian on this mode.
    lam = 12.0*np.sin(np.pi/n)**2/(h*h)
    rhs = lam*p_exact
    p = np.zeros_like(rhs)
    diag = 6.0/(h*h)
    for _ in range(iterations):
        neighbors = (np.roll(p,1,0)+np.roll(p,-1,0)+np.roll(p,1,1)+np.roll(p,-1,1)+np.roll(p,1,2)+np.roll(p,-1,2))/(h*h)
        p = (neighbors + rhs)/diag
        p -= p.mean()  # fix gauge
    lap = (np.roll(p,1,0)-2*p+np.roll(p,-1,0) + np.roll(p,1,1)-2*p+np.roll(p,-1,1) + np.roll(p,1,2)-2*p+np.roll(p,-1,2))/(h*h)
    residual = rhs + lap
    ratio = float(np.linalg.norm(residual.ravel()) / np.linalg.norm(rhs.ravel()))
    threshold = 1e-3
    return Result('pressure_residual_relative', ratio, threshold, ratio < threshold,
                  f'periodic manufactured Poisson problem, n={n}, Jacobi iterations={iterations}; residual-based criterion')

def projection_divergence_test(n: int = 20, iterations: int = 500) -> Result:
    # Constant-density periodic projection with collocated arrays and consistent
    # backward divergence/forward gradient pairing.
    x = 2*np.pi*np.arange(n)/n; h=2*np.pi/n
    X,Y,Z=np.meshgrid(x,x,x,indexing='ij')
    u=np.sin(X)*np.cos(Y); v=np.cos(X)*np.sin(Y); w=np.sin(Z)*0.0
    div=(u-np.roll(u,1,0))/h+(v-np.roll(v,1,1))/h+(w-np.roll(w,1,2))/h
    before=float(np.sqrt(np.mean(div*div)))
    # Solve Lp=div (periodic, zero mean), then subtract forward pressure gradient.
    p=np.zeros_like(div); rhs=div/h  # scale is immaterial for reduction; correct below via operator
    # Standard Jacobi for lap(p)=div, with coefficient 1/h^2.
    rhs=div
    for _ in range(iterations):
        neighbors=(np.roll(p,1,0)+np.roll(p,-1,0)+np.roll(p,1,1)+np.roll(p,-1,1)+np.roll(p,1,2)+np.roll(p,-1,2))
        p=(neighbors-h*h*rhs)/6.0
        p-=p.mean()
    # D(G(p)) matches the backward divergence of forward-gradient face update.
    u2=u-(np.roll(p,-1,0)-p)/h
    v2=v-(np.roll(p,-1,1)-p)/h
    w2=w-(np.roll(p,-1,2)-p)/h
    div2=(u2-np.roll(u2,1,0))/h+(v2-np.roll(v2,1,1))/h+(w2-np.roll(w2,1,2))/h
    after=float(np.sqrt(np.mean(div2*div2)))
    ratio=after/max(before,1e-30)
    threshold=0.05
    return Result('projection_divergence_ratio',ratio,threshold,ratio<threshold,
                  f'periodic constant-density manufactured projection, n={n}, Jacobi iterations={iterations}; after/before RMS divergence')

def conservative_transport_mass_test(n: int = 128, steps: int = 100) -> Result:
    # Periodic finite-volume flux difference; uniform velocity means exact telescoping.
    dx=1.0/n; x=(np.arange(n)+0.5)*dx
    q=1.0+0.25*np.sin(2*np.pi*x)
    u=0.4; dt=0.2*dx/abs(u)
    mass0=float(q.sum()*dx)
    for _ in range(steps):
        # Upwind conservative face flux for positive u.
        flux=u*np.roll(q,1)
        q=q-dt/dx*(u*q-flux)
    drift=abs(float(q.sum()*dx)-mass0)/abs(mass0)
    threshold=1e-12
    return Result('periodic_scalar_mass_relative_drift',drift,threshold,drift<threshold,
                  f'finite-volume upwind transport, n={n}, steps={steps}; periodic closed domain')

def advection_temporal_convergence() -> Result:
    # Linear advection on periodic domain using SSP-RK2 + centered spatial derivative.
    # dt proportional to dx^2 isolates second-order spatial/time convergence robustly.
    errors=[]
    for n in (32,64,128):
        dx=1.0/n; x=(np.arange(n)+0.5)*dx; u=0.5; final_t=0.1
        q=np.sin(2*np.pi*x); dt=min(0.15*dx/abs(u), 0.2*dx*dx)
        count=math.ceil(final_t/dt); dt=final_t/count
        def rhs(a): return -u*(np.roll(a,-1)-np.roll(a,1))/(2*dx)
        for _ in range(count):
            r0=rhs(q); pred=q+dt*r0; r1=rhs(pred); q=0.5*(q+pred+dt*r1)
        exact=np.sin(2*np.pi*(x-u*final_t))
        errors.append(float(np.sqrt(np.mean((q-exact)**2))))
    order=math.log(errors[0]/errors[1],2)
    order2=math.log(errors[1]/errors[2],2)
    observed=min(order,order2)
    threshold=1.7
    return Result('advection_refinement_order_min',observed,threshold,observed>threshold,
                  f'periodic smooth solution; L2 errors={errors}; observed orders=({order:.3f},{order2:.3f})')

def advection_timestep_convergence() -> Result:
    # Isolate SSP-RK2 temporal order using a smooth linear mode q'=-lambda*q.
    lam=2.0; final_t=1.0; exact=math.exp(-lam*final_t); errors=[]
    for nsteps in (20,40,80):
        dt=final_t/nsteps; q=1.0
        for _ in range(nsteps):
            r0=-lam*q; predictor=q+dt*r0; r1=-lam*predictor
            q=0.5*(q+predictor+dt*r1)
        errors.append(abs(q-exact))
    order1=math.log(errors[0]/errors[1],2); order2=math.log(errors[1]/errors[2],2)
    observed=min(order1,order2); threshold=1.8
    return Result('ssp_rk2_timestep_order_min',observed,threshold,observed>threshold,
                  f'linear decay manufactured solution; errors={errors}; observed orders=({order1:.3f},{order2:.3f})')

def scalar_grid_refinement() -> Result:
    # Spatial refinement at fixed very small CFL/time integration error.
    errors=[]
    for n in (32,64,128):
        dx=1.0/n; x=(np.arange(n)+0.5)*dx; u=0.5; final_t=0.1
        q=np.sin(2*np.pi*x); dt0=0.02*dx/abs(u)
        count=math.ceil(final_t/dt0); dt=final_t/count
        def rhs(a): return -u*(np.roll(a,-1)-np.roll(a,1))/(2*dx)
        for _ in range(count):
            r0=rhs(q); pred=q+dt*r0; r1=rhs(pred); q=0.5*(q+pred+dt*r1)
        exact=np.sin(2*np.pi*(x-u*final_t))
        errors.append(float(np.sqrt(np.mean((q-exact)**2))))
    order1=math.log(errors[0]/errors[1],2); order2=math.log(errors[1]/errors[2],2)
    observed=min(order1,order2); threshold=1.7
    return Result('advection_grid_refinement_order_min',observed,threshold,observed>threshold,
                  f'fixed low CFL; L2 errors={errors}; observed orders=({order1:.3f},{order2:.3f})')

def main() -> int:
    results=[poisson_jacobi_test(), projection_divergence_test(), conservative_transport_mass_test(), advection_temporal_convergence(), advection_timestep_convergence(), scalar_grid_refinement()]
    payload={'suite':'SOS independent numerical reference checks','python':sys.version.split()[0], 'platform':platform.platform(),
             'numpy':np.__version__, 'scope_warning':'These tests execute independent Python reference operators; they do NOT compile or execute SOS Rust/Metal source.',
             'results':[asdict(r) for r in results], 'passed':sum(r.passed for r in results), 'failed':sum(not r.passed for r in results)}
    print(json.dumps(payload,indent=2))
    return 0 if payload['failed']==0 else 1
if __name__=='__main__': raise SystemExit(main())
