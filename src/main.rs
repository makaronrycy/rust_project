use std::env;


mod common;

/*sources:
https://whoisryosuke.com/blog/2022/render-pipelines-in-wgpu-and-rust#multiple-models
sotrh.github.io/learn-wgpu/intermediate/

*/
fn main(){
    
    env::set_var("RUST_BACKTRACE", "1");
    common::run("Bowling");
}

