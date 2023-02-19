pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}

pub enum ColliderShape {
    Ball(f32),
    Box(f32, f32, f32),
    Cylinder(f32, f32),
    Capsule(f32, f32),
    Convex,
    Concave,
}

pub enum ColliderType {
    Collider,
    Trigger,
}
