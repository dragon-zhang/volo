//! [`Extension`] support for inserting or extracting anything for contexts
use motore::{layer::Layer, service::Service};
use volo::context::Context;

#[cfg(feature = "server")]
mod server;

/// Inserting anything into contexts as a [`Layer`] or extracting anything as an extractor
///
/// # Examples
///
/// ```
/// use volo_http::{
///     extension::Extension,
///     server::route::{get, Router},
/// };
///
/// #[derive(Clone)]
/// struct State {
///     foo: String,
/// }
///
/// // A handler for extracting the `State` from `Extension`
/// async fn show_state(Extension(state): Extension<State>) -> String {
///     state.foo
/// }
///
/// let router: Router = Router::new()
///     .route("/", get(show_state))
///     // Use `Extension` as a `Layer`
///     .layer(Extension(State {
///         foo: String::from("bar"),
///     }));
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Extension<T>(pub T);

impl<S, T> Layer<S> for Extension<T>
where
    S: Send + Sync + 'static,
    T: Sync,
{
    type Service = ExtensionService<S, T>;

    fn layer(self, inner: S) -> Self::Service {
        ExtensionService { inner, ext: self.0 }
    }
}

/// A [`Service`] generated by [`Extension`] as a [`Layer`] for inserting something into Contexts.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExtensionService<I, T> {
    inner: I,
    ext: T,
}

impl<S, Cx, Req, Resp, E, T> Service<Cx, Req> for ExtensionService<S, T>
where
    S: Service<Cx, Req, Response = Resp, Error = E> + Send + Sync + 'static,
    Req: Send,
    Cx: Context + Send,
    T: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut Cx,
        req: Req,
    ) -> Result<Self::Response, Self::Error> {
        cx.extensions_mut().insert(self.ext.clone());
        self.inner.call(cx, req).await
    }
}
