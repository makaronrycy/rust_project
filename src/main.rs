use vertex_data::{cube_normals, cube_positions};
use std::env;


mod common;
mod vertex_data;
/* 
fn create_sphere(r: f32, u:usize, v: usize) -> Vec<common::Vertex> {
    let(pos, normal, _uvs) = vertex_data::sphere_data(r, u, v);
    let mut data:Vec<common::Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(common::vertex(pos[i], normal[i]));
    }
    data.to_vec()
}
fn create_cube() -> Vec<common::Vertex> {
    let pos = cube_positions();
    let normal = cube_normals();
    let mut data:Vec<common::Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(common::vertex(pos[i], normal[i]));
    }
    data.to_vec()
}
*/

fn main(){
    
    /*
    let sphere_data1 = create_sphere(1.5, 15, 20);
    let cube_data = create_cube();
     
    vertex_datas.push(sphere_data1);
    vertex_datas.push(cube_data);
   */
    env::set_var("RUST_BACKTRACE", "1");
    common::run("Bowling");
}

