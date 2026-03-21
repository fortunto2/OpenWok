use crate::dispatch::{OrderEvent, auto_dispatch};
use crate::order::{Order, OrderStatus};
use crate::repo::{CreateOrderRequest, RepoError, Repository};
use crate::types::OrderId;

pub struct TransitionOrderResult {
    pub order: Order,
    pub events: Vec<OrderEvent>,
}

pub async fn create_order<R: Repository>(
    repo: &R,
    req: CreateOrderRequest,
) -> Result<Order, RepoError> {
    repo.create_order(req).await
}

pub async fn get_order<R: Repository>(repo: &R, id: OrderId) -> Result<Order, RepoError> {
    repo.get_order(id).await
}

pub async fn transition_order<R: Repository>(
    repo: &R,
    id: OrderId,
    status: OrderStatus,
) -> Result<TransitionOrderResult, RepoError> {
    let order = repo.update_order_status(id, status).await?;
    let mut events = vec![OrderEvent {
        order_id: order.id.to_string(),
        status: format!("{:?}", order.status),
    }];

    if order.status == OrderStatus::ReadyForPickup
        && let Some(result) = auto_dispatch(repo, order.id).await?
    {
        let updated = repo
            .update_order_status(order.id, OrderStatus::InDelivery)
            .await?;
        events.push(OrderEvent {
            order_id: result.order_id.clone(),
            status: "CourierAssigned".into(),
        });
        events.push(OrderEvent {
            order_id: result.order_id,
            status: "InDelivery".into(),
        });

        return Ok(TransitionOrderResult {
            order: updated,
            events,
        });
    }

    Ok(TransitionOrderResult { order, events })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::application::test_support::TestRepo;

    #[tokio::test]
    async fn ready_for_pickup_auto_dispatches_and_emits_assignment_events() {
        let (repo, order_id) = TestRepo::with_preparing_order_and_available_courier();

        let result = transition_order(&repo, order_id, OrderStatus::ReadyForPickup)
            .await
            .unwrap();

        assert_eq!(result.order.status, OrderStatus::InDelivery);
        assert!(result.order.courier_id.is_some());
        assert_eq!(result.events.len(), 3);
        assert_eq!(result.events[0].status, "ReadyForPickup");
        assert_eq!(result.events[1].status, "CourierAssigned");
        assert_eq!(result.events[2].status, "InDelivery");
    }
}
