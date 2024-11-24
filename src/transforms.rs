//source: https://github.com/jack1232/wgpu11
#![allow(dead_code)]
use std::f32::consts::PI;

use cgmath::*;

#[rustfmt::skip]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);


pub struct Transform{
    //x,y,z
    trans_mat: Matrix4<f32>,
    rotate_mat_x: Matrix4<f32>,
    rotate_mat_y: Matrix4<f32>,
    rotate_mat_z: Matrix4<f32>,
    scale_mat: Matrix4<f32>
}

impl Transform{
    pub fn new(translation:[f32; 3], rotation:[f32; 3], scaling:[f32; 3]) ->Self{
        // create transformation matrices
        let trans_mat = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
        let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
        let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
        let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
        let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);
        Self{
            trans_mat,
            rotate_mat_x,
            rotate_mat_y,
            rotate_mat_z,
            scale_mat
        }
    }
    pub fn rotate(&mut self,rotation:[f32; 3]) -> Matrix4<f32>{
        self.rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
        self.rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
        self.rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
        self.get_model_mat()
    }
    pub fn translate(&mut self,translation:[f32; 3]) -> Matrix4<f32>{
        self.trans_mat = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
        self.get_model_mat()
    }
    pub fn scale(&mut self,scaling:[f32; 3])->Matrix4<f32>{
        self.scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);
        self.get_model_mat()
    }
    pub fn get_model_mat(&mut self) -> Matrix4<f32>{
        let model_mat = self.trans_mat * self.rotate_mat_z * self.rotate_mat_y * self.rotate_mat_x * self.scale_mat;
        model_mat
    }
}

pub fn create_view(camera_position: Point3<f32>, look_direction: Point3<f32>, up_direction: Vector3<f32>) -> Matrix4<f32> {
    Matrix4::look_at_rh(camera_position, look_direction, up_direction)
}

pub fn create_projection(aspect:f32, is_perspective:bool) -> Matrix4<f32> {
    let project_mat:Matrix4<f32>;
    if is_perspective {
        project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.1, 100.0);
    } else {
        project_mat = OPENGL_TO_WGPU_MATRIX * ortho(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0);
    }
    project_mat
}

pub fn create_perspective_projection(fovy:Rad<f32>, aspect:f32, near: f32, far:f32) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(fovy, aspect, near, far)
}

pub fn create_projection_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far)    
}

pub fn create_view_projection_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32, camera_position: Point3<f32>, 
    look_direction: Point3<f32>, up_direction: Vector3<f32>) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    
    // construct view matrix
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);     

    // construct projection matrix
    let project_mat = OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far);    
    
    // contruct view-projection matrix
    let view_project_mat = project_mat * view_mat;
   
    // return various matrices
    (view_mat, project_mat, view_project_mat)
} 

pub fn create_view_projection(camera_position: Point3<f32>, look_direction: Point3<f32>, up_direction: Vector3<f32>,
    aspect:f32, is_perspective:bool) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    
    // construct view matrix
    let view_mat = Matrix4::look_at_rh(camera_position, look_direction, up_direction);     

    // construct projection matrix
    let project_mat:Matrix4<f32>;
    if is_perspective {
        project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.1, 100.0);
    } else {
        project_mat = OPENGL_TO_WGPU_MATRIX * ortho(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0);
    }
    
    // contruct view-projection matrix
    let view_project_mat = project_mat * view_mat;
   
    // return various matrices
    (view_mat, project_mat, view_project_mat)
} 

pub fn create_transforms(translation:[f32; 3], rotation:[f32; 3], scaling:[f32; 3]) -> Matrix4<f32> {

    // create transformation matrices
    let trans_mat = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
    let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
    let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
    let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
    let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);

    // combine all transformation matrices together to form a final transform matrix: model matrix
    let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;

    // return final model matrix
    model_mat
}