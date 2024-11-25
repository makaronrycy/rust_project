
use rapier3d::na::vector;
use winit::keyboard::KeyCode;
use winit::event::*;

use super::{camera::{ Camera, CameraController}, phys::Physics};
pub fn process_keyboard(key: KeyCode, state: ElementState, camera_controller:&mut CameraController,physics: &mut Physics,camera: &Camera) -> bool {
    let amount = if state == ElementState::Pressed {
        1.0
    } else {
        0.0
    };
    match key {
        KeyCode::KeyW | KeyCode::ArrowUp => {
            camera_controller.forward(amount);
            true
        }
        KeyCode::KeyS | KeyCode::ArrowDown => {
            camera_controller.backward(amount);
            true
        }
        KeyCode::KeyA | KeyCode::ArrowLeft => {
            camera_controller.left(amount);
            true
        }
        KeyCode::KeyD | KeyCode::ArrowRight => {
            camera_controller.right(amount);
            true
        }
        KeyCode::Space => {
            camera_controller.up(amount);
            true
        }
        KeyCode::ShiftLeft => {
            camera_controller.down(amount);
            true
        }
        KeyCode::KeyE =>{
            
            physics.throw_ball(camera.look_direction());
            true
        }
        KeyCode::KeyV =>{
            
            physics.reset_ball_vel();
            true
        }
        _ => false,
    }
}