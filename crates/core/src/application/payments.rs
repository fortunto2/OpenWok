use crate::order::{Order, OrderStatus};
use crate::repo::{RepoError, Repository};
use crate::types::{
    CreatePaymentRequest, OrderId, Payment, PaymentStatus, UpdatePaymentStatusRequest,
};

pub async fn create_payment_for_order<R: Repository>(
    repo: &R,
    order: &Order,
) -> Result<Payment, RepoError> {
    repo.create_payment(CreatePaymentRequest {
        order_id: order.id,
        stripe_checkout_session_id: None,
        amount_total: order.pricing.total(),
        restaurant_amount: order.pricing.food_total,
        courier_amount: order.pricing.delivery_fee + order.pricing.tip,
        federal_amount: order.pricing.federal_fee,
        local_ops_amount: order.pricing.local_ops_fee,
        processing_amount: order.pricing.processing_fee,
    })
    .await
}

pub async fn mark_payment_succeeded<R: Repository>(
    repo: &R,
    order_id: OrderId,
    stripe_payment_intent_id: Option<String>,
) -> Result<(), RepoError> {
    let payment = repo.get_payment_by_order(order_id).await?;

    if payment.status != PaymentStatus::Succeeded {
        repo.update_payment_status(
            payment.id,
            UpdatePaymentStatusRequest {
                status: PaymentStatus::Succeeded,
                stripe_payment_intent_id,
            },
        )
        .await?;
    }

    let order = repo.get_order(order_id).await?;
    if order.status == OrderStatus::Created {
        repo.update_order_status(order_id, OrderStatus::Confirmed)
            .await?;
    }

    Ok(())
}

pub async fn mark_payment_failed<R: Repository>(
    repo: &R,
    order_id: OrderId,
) -> Result<(), RepoError> {
    let payment = repo.get_payment_by_order(order_id).await?;

    if payment.status != PaymentStatus::Failed {
        repo.update_payment_status(
            payment.id,
            UpdatePaymentStatusRequest {
                status: PaymentStatus::Failed,
                stripe_payment_intent_id: None,
            },
        )
        .await?;
    }

    let order = repo.get_order(order_id).await?;
    if order.status == OrderStatus::Created {
        repo.update_order_status(order_id, OrderStatus::Cancelled)
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::application::test_support::TestRepo;

    #[tokio::test]
    async fn mark_payment_succeeded_is_idempotent_and_confirms_order() {
        let (repo, order_id) = TestRepo::with_created_order_and_payment(PaymentStatus::Pending);

        mark_payment_succeeded(&repo, order_id, Some("pi_123".into()))
            .await
            .unwrap();
        mark_payment_succeeded(&repo, order_id, Some("pi_456".into()))
            .await
            .unwrap();

        let payment = repo.payment_by_order(order_id);
        let order = repo.order(order_id);

        assert_eq!(payment.status, PaymentStatus::Succeeded);
        assert_eq!(payment.stripe_payment_intent_id.as_deref(), Some("pi_123"));
        assert_eq!(order.status, OrderStatus::Confirmed);
    }

    #[tokio::test]
    async fn mark_payment_failed_cancels_created_order() {
        let (repo, order_id) = TestRepo::with_created_order_and_payment(PaymentStatus::Pending);

        mark_payment_failed(&repo, order_id).await.unwrap();

        let payment = repo.payment_by_order(order_id);
        let order = repo.order(order_id);

        assert_eq!(payment.status, PaymentStatus::Failed);
        assert_eq!(order.status, OrderStatus::Cancelled);
    }
}
