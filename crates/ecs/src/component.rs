use std::any::Any;

/// Marker trait for data that can be stored as an ECS component.
/// 
/// Components should primarily contain data rather than behaviour.
pub trait Component: Any + Send + Sync + 'static {}

/// Automatically make any compatible type a Component
/// 
/// This means you do not need to manually implement Component
/// for every struct such as Position, Velocity or Health.
impl<T> Component for T
where
T: Any + Send + Sync + 'static,
{

}
