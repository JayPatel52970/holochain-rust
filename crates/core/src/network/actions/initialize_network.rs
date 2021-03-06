use crate::{
    action::{Action, ActionWrapper, NetworkSettings},
    context::{get_dna_and_agent, Context},
    instance::dispatch_action,
    network::handler::create_handler,
};
use futures::{task::Poll, Future};
use holochain_core_types::error::HcResult;
#[cfg(test)]
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
use std::{pin::Pin, sync::Arc};

/// Creates a network proxy object and stores DNA and agent hash in the network state.
#[autotrace]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn initialize_network(context: &Arc<Context>) -> HcResult<()> {
    let (dna_address, agent_id) = get_dna_and_agent(context).await?;
    let handler = create_handler(&context, dna_address.to_string());
    let network_settings = NetworkSettings {
        p2p_config: context.p2p_config.clone(),
        dna_address,
        agent_id: agent_id.clone(),
        handler,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    log_debug!(context, "waiting for network");
    let id = ProcessUniqueId::new();
    InitNetworkFuture {
        context: context.clone(),
        id,
    }
    .await?;

    Ok(())
}

#[cfg(test)]
pub async fn initialize_network_with_spoofed_dna(
    dna_address: Address,
    context: &Arc<Context>,
) -> HcResult<()> {
    let (_, agent_id) = get_dna_and_agent(context).await?;
    let handler = create_handler(&context, dna_address.to_string());
    let network_settings = NetworkSettings {
        p2p_config: context.p2p_config.clone(),
        dna_address,
        agent_id,
        handler,
    };
    let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let id = ProcessUniqueId::new();
    InitNetworkFuture {
        context: context.clone(),
        id,
    }
    .await
}

pub struct InitNetworkFuture {
    context: Arc<Context>,
    id: ProcessUniqueId,
}

impl Future for InitNetworkFuture {
    type Output = HcResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("InitializeNetworkFuture") {
            return Poll::Ready(Err(err));
        }

        self.context
            .register_waker(self.id.clone(), cx.waker().clone());

        if let Some(state) = self.context.try_state() {
            if state.network().network.is_some()
                && state.network().dna_address.is_some()
                && state.network().agent_id.is_some()
            {
                self.context.unregister_waker(self.id.clone());
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
