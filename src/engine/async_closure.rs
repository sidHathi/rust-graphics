use super::component::Component;

pub async fn run_component_closure<F, Fut>(closure: F, component: &mut Component) -> ()
where
    F: FnOnce(&mut Component) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    closure(component).await
}
