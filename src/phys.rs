use std::{fmt::format, ops::Index};
use cgmath::{self};
use rapier3d::prelude::*;
use nalgebra::{Vector3, vector, Vector};
use winit::event::Force;

pub struct PhysicsObj{
    name: String,
    handle: RigidBodyHandle,
}
pub struct Physics{
    gravity: Vector3<f32>,
    physics_pipeline: PhysicsPipeline,
    integration_params: IntegrationParameters,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    island_manager: IslandManager,
    query_pipeline: QueryPipeline,
    physics_obj: Vec<PhysicsObj>
}
impl Physics{
    pub fn new() -> Self{
        let gravity = vector![0.0, -9.81, 0.0];
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joints: ImpulseJointSet = ImpulseJointSet::new();
        let multibody_joints = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let island_manager = IslandManager::new();
        let integration_params=IntegrationParameters::default();
        let query_pipeline = QueryPipeline::new();
        let physics_obj: Vec<PhysicsObj> = Vec::new();
        Self { 
            gravity,
            physics_pipeline,
            bodies, 
            colliders,
            broad_phase,
            narrow_phase,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            integration_params,
            island_manager,
            query_pipeline,
            physics_obj
        }   
    }
    pub fn simulate(&mut self){
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
        
    }
    pub fn get_translation(&mut self,index: usize) ->[f32;3]{
        
        let trans_obj = &self.bodies.get(self.physics_obj[index].handle).unwrap().translation();
        let translation =  [trans_obj.x,trans_obj.y,trans_obj.z];
        //if(index == 1) {println!("Position: {},{},{}",trans_obj.x,trans_obj.y,trans_obj.z)};
        translation
        
    }
    pub fn get_rotation(&mut self,index: usize) ->[f32;3]{
        let rotation_obj = &self.bodies.get(self.physics_obj[index].handle).unwrap().rotation();
        let rotation = [rotation_obj.i,rotation_obj.j,rotation_obj.k] ;
        //if(index == 1) {println!("Rotation{},{},{}",rotation_obj.i,rotation_obj.j,rotation_obj.k)};
        rotation
    }
    pub fn throw_ball(&mut self,force:Vector3<f32>){
        println!("{}",self.physics_obj[0].name);
        //ball is always on index 0
        let body = self.bodies.get_mut(self.physics_obj[0].handle).unwrap();
        body.apply_impulse(force*0.2, true);
    }
    pub fn reset_ball_vel(&mut self){
        let body = self.bodies.get_mut(self.physics_obj[0].handle).unwrap();
        //it's waken up when we throw the ball again
        body.sleep();
    }
    pub fn build_colliders(&mut self){

        let bowling_body = RigidBodyBuilder::dynamic()
        .translation(vector![-4.5,1.0,4.5])
        .sleeping(true)
        .linear_damping(0.01)
        .angular_damping(0.01)
        .build();
        let bowling_handle = self.bodies.insert(bowling_body);
        self.physics_obj.push(PhysicsObj{name: "bowling_ball".to_string(),handle: bowling_handle});
        let bowling_collider = ColliderBuilder::ball(0.2)
        .restitution(0.1)
        .density(10.0)
        .friction(0.5)
        .build();
        self.colliders.insert_with_parent(bowling_collider, bowling_handle, &mut self.bodies);
        
        // Create pins (10 pins in triangle formation)
        for row in 0..5 {
            let z = row as f32 * 0.8;
            let x_off=-(row as f32 )* 0.8 /2.0;
            for col in 0..row{
                let x = x_off + col as f32 * 0.8;
                let pin_body = RigidBodyBuilder::dynamic()
                    .translation(vector![x, 0.0, z])
                    .angular_damping(1.0)
                    .linear_damping(1.0)
                    //.sleeping(true)
                    .build();
            let pin_handle = self.bodies.insert(pin_body);
            self.physics_obj.push(PhysicsObj{name: format!("Pin{row},{col}"),handle: pin_handle});
            let pin_collider = ColliderBuilder::cylinder(0.38, 0.05)
                .restitution(0.9)
                .friction(0.4)
                .density(2.0)
                .build();
            self.colliders.insert_with_parent(pin_collider, pin_handle, &mut self.bodies);
        
            }
            }
        for (i,obj) in self.physics_obj.iter().enumerate(){
            let name = &obj.name;
            println!("Index: {i}, Name: {name}");
        }
        let floor_body = RigidBodyBuilder::fixed().translation(vector![0.0,-1.0,0.0]).build();
        let floor_handle = self.bodies.insert(floor_body);
        let floor_collider = ColliderBuilder::cuboid(100.0, 1.0, 100.0)
        .friction(0.1)
        .restitution(0.0)
        .build();
        self.colliders.insert_with_parent(floor_collider, floor_handle, &mut self.bodies);
        self.physics_obj.push(PhysicsObj{name: format!("Floor"),handle: floor_handle});

    }
}