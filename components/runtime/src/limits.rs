use std::{cell::RefCell, num::NonZeroU32, rc::Rc};

use crate::RuntimeContext;
use deno_core::OpState;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota,
};
use stores::config::PremiumSlotTier;

pub type RateLimiter = governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

macro_rules! ratelimits {
    ($($name:ident => [$none:literal, $lite:literal, $premium:literal]),*) => {
        pub struct RateLimiters {
            $($name: RateLimiter,)*
        }

        impl RateLimiters {
            $(pub async fn $name(op_state: &Rc<RefCell<OpState>>) {
                let ratelimiters = { op_state.borrow().borrow::<Rc<RateLimiters>>().clone() };
                ratelimiters.$name.until_ready().await;
            })*
        }

        impl RateLimiters {
            pub fn new(tier: Option<PremiumSlotTier>) -> Self {
                Self {
                    $(
                        $name: RateLimiter::direct(Quota::per_second(
                            NonZeroU32::new(match tier {
                                None => $none,
                                Some(PremiumSlotTier::Lite) => $lite,
                                Some(PremiumSlotTier::Premium) => $premium,
                            })
                            .unwrap(),
                        )),
                    )*
                }
            }
        }
    };
}

macro_rules! numeric_limit {
    ($name:ident => [$none:literal, $lite:literal, $premium:literal]) => {
        pub fn $name(op_state: &Rc<RefCell<OpState>>) -> u64 {
            let premium_tier = {
                let state = op_state.borrow();
                state.borrow::<RuntimeContext>().premium_tier
            };

            match premium_tier {
                None => $none,
                Some(PremiumSlotTier::Lite) => $lite,
                Some(PremiumSlotTier::Premium) => $premium,
            }
        }
    };
}

ratelimits! {
    // number of guild http requests per second
    user_http => [1, 2, 2],
    // number of task operations per second
    task_ops => [1, 2, 3],

    // number of times we can fetch a public discord invite,
    // needed because this endpoint is not guild scoped
    discord_get_public_invite => [1, 1, 1]
}

// max total amount of bucket storage used on a guild
numeric_limit! {storage_total_size => [1_000_000, 10_000_000, 100_000_000]}

// max data size in a single task
numeric_limit! {tasks_data_size => [1_000, 10_000, 10_000]}

// max number of scheduled tasks
numeric_limit! {tasks_scheduled_count => [10_000, 100_000, 100_000]}
