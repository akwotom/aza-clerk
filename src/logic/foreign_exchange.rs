/*
    Copyright 2026 Son of Binary
    The aza-clerk Project

    This module contains the logic for handling foreign exchange currency conversions.
*/

use crate::models::Amount;

/// This method computes the equivalent of the given amount in the stated currency.
pub async fn convert(from: Amount, to: String) -> i32 {
    if (from.currency == to) {
        return from.value;
    }

    if from.currency == "USD" && to == "XAF" {
        return from.value * 570;
    }

    if from.currency == "XAF" && to == "USD" {
        return from.value / 570;
    };

    panic!("For now, currency conversion only happens USD <-> XAF.");
}
