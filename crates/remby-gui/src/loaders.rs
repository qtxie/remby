use gpui::*;

pub fn spawn_async<T, F, R>(
    entity: &Entity<T>,
    cx: &mut Context<T>,
    future: F,
    on_result: impl FnOnce(&mut T, &mut Context<T>, R) + 'static,
) where
    T: 'static,
    F: std::future::Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::tokio_runtime().spawn(async move {
        let result = future.await;
        let _ = tx.send(result);
    });
    let entity = entity.clone();
    cx.spawn(async move |_window, cx| {
        if let Ok(result) = rx.await {
            entity.update(cx, |app, cx| {
                on_result(app, cx, result);
            });
        }
    })
    .detach();
}
