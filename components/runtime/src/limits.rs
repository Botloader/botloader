use std::{cell::RefCell, num::NonZeroU32, rc::Rc};

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

ratelimits! {
    user_http => [1, 2, 2],
    task_ops => [1, 2, 3]
}
