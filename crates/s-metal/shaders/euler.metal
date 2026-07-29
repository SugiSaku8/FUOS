#include <metal_stdlib>

using namespace metal;


kernel void euler_step(

    device const float* current
        [[buffer(0)]],

    device const float* derivative
        [[buffer(1)]],

    device float* output
        [[buffer(2)]],


    constant float& dt
        [[buffer(3)]],


    uint id
        [[thread_position_in_grid]]

)
{

    output[id] =
        current[id]
        +
        dt * derivative[id];

}
