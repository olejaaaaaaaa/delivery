use glam::Vec3;
use rapier3d::{dynamics::{CCDSolver, GenericJoint, ImpulseJointHandle, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{BroadPhaseBvh, Collider, ColliderBuilder, ColliderHandle, ColliderSet, DefaultBroadPhase, NarrowPhase}, math::Vector, pipeline::PhysicsPipeline};

pub struct PhysicsEngine {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    gravity: Vec3,
    phys_pipeline: PhysicsPipeline,
    integration_params: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    pub impulse: ImpulseJointSet,
    multi: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsEngine {

    pub fn update(&mut self, dt: f32) {
        self.integration_params.dt = dt;
        self.phys_pipeline.step(
            self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse,
            &mut self.multi,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn create_joint(&mut self, rb1: RigidBodyHandle, rb2: RigidBodyHandle, data: impl Into<GenericJoint>, wake_up: bool) -> ImpulseJointHandle {
        self.impulse.insert(rb1, rb2, data, wake_up)
    }

    pub fn create_rigid_body(&mut self, body: RigidBody) -> RigidBodyHandle {
        self.rigid_body_set.insert(body)
    }

    pub fn get_collider(&self, collider: ColliderHandle) -> Option<&Collider> {
        self.collider_set.get(collider)
    }

    pub fn create_collider(&mut self, collider: Collider, body_handle: RigidBodyHandle) -> ColliderHandle {
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set)
    }

    pub fn new() -> Self {

        let rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // let floor = ColliderBuilder::cuboid(100.0, 0.01, 100.0)
        //     .translation(Vec3::new(0.0, 0.0, 0.0))
        //     .build();

        // collider_set.insert(floor);

        /* Create other structures necessary for the simulation. */
        let gravity = Vector::new(0.0, -9.81, 0.0);
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            rigid_body_set,
            collider_set,
            gravity,
            phys_pipeline: physics_pipeline,
            integration_params: integration_parameters,
            island_manager,
            impulse: impulse_joint_set,
            multi: multibody_joint_set,
            broad_phase,
            narrow_phase,
            ccd_solver
        }
    }
}

