use std::rc::Rc;

// Abstract over extracting values from Option<Rc<T>> and Rc<T>, so that we can
// do pointer comparison.
pub trait MaybeRc<T> {
    fn cache_is_some(&self) -> bool;
    fn are_contents_copyable(&self) -> bool;
    fn cache_unwrap(&self) -> &Rc<T>;
    fn unwrap_copyable_value(&self) -> T;
}

impl<T> MaybeRc<T> for Option<Rc<T>> {
    fn cache_is_some(&self) -> bool {
        self.is_some()
    }

    fn are_contents_copyable(&self) -> bool {
        false
    }

    fn unwrap_copyable_value(&self) -> T {
        panic!("Logic error, Rc<T> is not copyable.");
    }

    fn cache_unwrap(&self) -> &Rc<T> {
        &self
            .as_ref()
            .expect("Logic error, cache unwrapping failed.")
    }
}

impl<T: Copy> MaybeRc<T> for T {
    fn cache_is_some(&self) -> bool {
        true
    }

    fn are_contents_copyable(&self) -> bool {
        true
    }

    fn unwrap_copyable_value(&self) -> T {
        *self
    }

    fn cache_unwrap(&self) -> &Rc<T> {
        panic!("Logic error, should not unwrap a copyable type.");
    }
}

impl<T: Copy> MaybeRc<T> for Option<T> {
    fn cache_is_some(&self) -> bool {
        self.is_some()
    }

    fn are_contents_copyable(&self) -> bool {
        true
    }

    fn unwrap_copyable_value(&self) -> T {
        panic!("Logic error, Rc<T> is not copyable.");
    }

    fn cache_unwrap(&self) -> &Rc<T> {
        panic!("Logic error, should not unwrap a copyable type.");
    }
}

impl<T> MaybeRc<T> for Rc<T> {
    fn cache_is_some(&self) -> bool {
        true
    }

    fn are_contents_copyable(&self) -> bool {
        false
    }

    fn unwrap_copyable_value(&self) -> T {
        panic!("Logic error, Rc<T> is not copyable.");
    }

    fn cache_unwrap(&self) -> &Rc<T> {
        self
    }
}

// Create a selector
#[macro_export]
macro_rules! selector {
    (
        pub fn $fn_name:ident(state: $State:ty) -> $Returns:ty {
            memoize |
                $( $selector_var_name:ident: $selector_fn_name:ident -> $SelectorReturns:ty ),*
            |
            $contents:tt
        }
    ) => { paste::paste! {
        pub fn $fn_name(state: $State) -> $Returns {
            // Call out to the $fn_name_selector_impl module for proper macro hygiene.
            [<$fn_name _selector_impl>]::$fn_name(state)
        }

        mod [<$fn_name _selector_impl>] {
            use super::*;
            use std::rc::Rc;
            use std::cell::RefCell;
            use std::option::Option;
            use std::borrow::Borrow;
            use crate::state::utils::MaybeRc;

            // e.g. the tuple: (ArgTypeA, ArgTypeB)
            type ArgsCacheTuple = ( $( $SelectorReturns ),* );

            thread_local! {
                // The arguments cache:
                // ```
                // pub static SELECTOR_GET_STATE_ARGS_CACHE: RefCell<Option<(Arg1, Arg2)>>
                //     = RefCell::new(None);
                // ```
                pub static [<SELECTOR_ $fn_name:upper _ARGS_CACHE>]: RefCell<Option<ArgsCacheTuple>> = RefCell::new(None);

                // The returns cache:
                // ```
                // pub static SELECTOR_GET_STATE_RETURNS_CACHE: RefCell<Option<ReturnType>>
                //     = RefCell::new(None);
                // ```
                pub static [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>]: RefCell<Option<$Returns>> = RefCell::new(None);
            }

            #[inline]
            pub fn $fn_name(state: $State) -> $Returns {
                // Name gross these gross invocations.
                let ref args_cache = [<SELECTOR_ $fn_name:upper _ARGS_CACHE>];
                let ref returns_cache = [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>];

                // Get the selector values.
                // let value_a = get_value_b(state.clone());
                // let value_b = get_value_b(state.clone());
                $(
                    let $selector_var_name = $selector_fn_name(state.clone());
                )*

                // See if the cached args match.
                let mut cache_matches = true;
                args_cache.with(|f| {
                    // Get each variable out of the cache.
                    if let Some((
                        $( ref [<cached_ $selector_var_name>] ),*
                    )) = *f.borrow() {
                        $(
                            if cache_matches {
                                if (
                                    // The emptiness doesn't match.
                                    $selector_var_name.cache_is_some() != [<cached_ $selector_var_name>].cache_is_some()
                                ) || (
                                    // There is a value to consider here.
                                    $selector_var_name.cache_is_some() &&

                                    // Determine how to compare the values.
                                    if $selector_var_name.are_contents_copyable() {
                                        // This is something like i64 or Option<i64>,
                                        // directly compare the values.
                                        $selector_var_name.unwrap_copyable_value()
                                        != [<cached_ $selector_var_name>].unwrap_copyable_value()
                                    } else {
                                        // Check the pointers for equality.
                                        !Rc::ptr_eq(
                                            $selector_var_name.cache_unwrap(),
                                            [<cached_ $selector_var_name>].cache_unwrap()
                                        )
                                    }

                                ){
                                    cache_matches = false;
                                }
                            }
                        )*
                    } else {
                        cache_matches = false;
                    }

                    if !cache_matches {
                        // Update the cached args by cloning the Rc.
                        let new_cache: ArgsCacheTuple = (
                            $( $selector_var_name.clone() ),*
                        );
                        *f.borrow_mut() = Some(new_cache);
                    }
                });

                // This is a cache hit, return from the cache.
                if cache_matches {
                    let mut result: Option<$Returns> = None;
                    returns_cache.with(|f| {
                        result = (*f.borrow()).clone();
                    });

                    #[cfg(feature = "selector-cache-log")]
                    println!("selector {} - cache hit", stringify!($fn_name));

                    return result.expect("Logic error, failed to get returns from cache.");
                }


                let return_value: $Returns = [<selector_ $fn_name _impl>]($( $selector_var_name ),*);

                let ref returns_cache = [<SELECTOR_ $fn_name:upper _RETURNS_CACHE>];
                returns_cache.with(|f| {
                    *f.borrow_mut() = Some(return_value.clone());
                });

                #[cfg(feature = "selector-cache-log")]
                println!("selector {} - cache miss", stringify!($fn_name));

                return_value
            }

            pub fn [<selector_ $fn_name _impl>](
                $( $selector_var_name: $SelectorReturns ),*
            ) -> $Returns {
                $contents
            }
        }
    }};
}

/// Combine the reducers
///
/// TODO - Return the old state if nothing has changed.
#[macro_export]
macro_rules! combine_reducers {
    (
        $state:ty,
        $action:ident,
        {
            $( $member:ident: $reducer:ty ),*
        }
    ) => {
        $state {
            $( $member: $reducer(self.$member.clone(), $action) ),*
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::game::primitives::{BBox, Position, Size};

    use super::*;

    #[derive(Clone, Debug)]
    pub struct TestState {
        position: Rc<Position>,
        size: Rc<Size>,
    }

    fn get_position(state: Rc<TestState>) -> Rc<Position> {
        state.position.clone()
    }

    fn get_size(state: Rc<TestState>) -> Rc<Size> {
        state.size.clone()
    }

    selector!(
        pub fn get_bbox(state: Rc<TestState>) -> Rc<BBox<i32>> {
            memoize |
                top_left: get_position -> Rc<Position>,
                size: get_size -> Rc<Size>
            | {
                Rc::from(BBox {
                    top_left: *top_left,
                    size: *size,
                })
            }
        }
    );

    #[test]
    fn test_state() {
        // Create an initial state.
        let state = Rc::from(TestState {
            position: Rc::new(Position::new(1, 1)),
            size: Rc::new(Size::new(5, 7)),
        });

        // Everything should only have one reference count to it.
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state.position), 1);
        assert_eq!(Rc::strong_count(&state.size), 1);

        let position = get_position(state.clone());

        // Now position will have two reference counts, as one is in the thread local
        // storage cache.
        assert_eq!(*position, Position::new(1, 1));
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state.position), 2);
        assert_eq!(Rc::strong_count(&state.size), 1);

        let size = get_size(state.clone());

        // The same behavior works for size.
        assert_eq!(*size, Position::new(5, 7));
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state.position), 2);
        assert_eq!(Rc::strong_count(&state.size), 2);

        {
            // Create a locally scoped bbox.
            let bbox = get_bbox(state.clone());

            assert_eq!(
                *bbox,
                BBox {
                    top_left: Position::new(1, 1),
                    size: Size::new(5, 7),
                }
            );

            // The bbox derives from the position and size, so the pointers will
            // increment.
            assert_eq!(Rc::strong_count(&state), 1);
            assert_eq!(Rc::strong_count(&state.position), 3);
            assert_eq!(Rc::strong_count(&state.size), 3);
        }

        // Even after dropping the bbox, the memoized values live on.
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state.position), 3);
        assert_eq!(Rc::strong_count(&state.size), 3);

        // Get cached bbox.
        let bbox = get_bbox(state.clone());

        // The bbox can be derived from the existing data.
        assert_eq!(
            *bbox,
            BBox {
                top_left: Position::new(1, 1),
                size: Size::new(5, 7),
            }
        );

        // None of the Rc values were updated, as they all point to the same one.
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state.position), 3);
        assert_eq!(Rc::strong_count(&state.size), 3);

        // Getting another bbox, will still re-use the cached version.
        assert!(std::rc::Rc::ptr_eq(&bbox, &get_bbox(state.clone())));

        // Create an equal but cloned state.
        let state_2 = Rc::from((*state).clone());
        assert_eq!(Rc::strong_count(&state), 1);
        assert_eq!(Rc::strong_count(&state_2), 1);
        assert_eq!(Rc::strong_count(&state.position), 4, "Incremented");
        assert_eq!(Rc::strong_count(&state.size), 4, "Incremented");

        // The cache is still valid.
        assert!(std::rc::Rc::ptr_eq(&bbox, &get_bbox(state_2.clone())));

        let state_3 = Rc::from(TestState {
            position: Rc::new(Position::new(0, 0)),
            size: state.size.clone(),
        });

        assert_eq!(Rc::strong_count(&state.position), 4);
        assert_eq!(Rc::strong_count(&state_3.position), 1);
        assert_eq!(Rc::strong_count(&state.size), 5, "Incremented");

        let bbox_3 = get_bbox(state_3.clone());
        assert_eq!(
            *bbox_3,
            BBox {
                top_left: Position::new(0, 0),
                size: Size::new(5, 7),
            }
        );

        assert_eq!(Rc::strong_count(&state.position), 3, "Decremented");
        assert_eq!(Rc::strong_count(&state_3.position), 2, "Incremented");
        assert_eq!(Rc::strong_count(&state.size), 5);

        assert_ne!(*bbox, *bbox_3, "The two are not equivalent.");
        assert!(
            !std::rc::Rc::ptr_eq(&bbox, &bbox_3),
            "Their pointers do not match."
        );
        assert!(
            std::rc::Rc::ptr_eq(&bbox_3, &get_bbox(state_3.clone())),
            "Re-running the selector matches the previous run."
        );

        // Now go back to the original bbox.
        let bbox_1 = get_bbox(state.clone());
        assert_eq!(bbox, bbox_1, "The original boxes are still equivalent");
        assert!(
            !std::rc::Rc::ptr_eq(&bbox, &bbox_1),
            "But the pointers are now different as it was re-computed."
        );
    }
}
