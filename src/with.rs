use std::future::Future;
use crate::Context;

#[async_trait::async_trait]
pub trait With<Output> {
    async fn with<Ctx>(self, ctx: Ctx) -> crate::Result<Output>
    where
        Ctx: Context + Send;
}

#[async_trait::async_trait]
impl<'a, Output, Fut> With<Output> for Fut
where
    Fut: Future<Output = Output> + Send + 'a
{
    async fn with<Ctx>(self, ctx: Ctx) -> crate::Result<Output>
    where
        Ctx: Context + Send,
    {
        ctx.handle(self).await
    }
}
