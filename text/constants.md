# Configurable Pallet Constants

`pallets/constant-config`
<a target="_blank" href="https://playground.substrate.dev/?deploy=recipes&files=%2Fhome%2Fsubstrate%2Fworkspace%2Fpallets%2Fconstant-config%2Fsrc%2Flib.rs">
	<img src="https://img.shields.io/badge/Playground-Try%20it!-brightgreen?logo=Parity%20Substrate" alt ="Try on playground"/>
</a>
<a target="_blank" href="https://github.com/substrate-developer-hub/recipes/tree/master/pallets/constant-config/src/lib.rs">
	<img src="https://img.shields.io/badge/Github-View%20Code-brightgreen?logo=github" alt ="View on GitHub"/>
</a>

To declare constant values within a runtime, it is necessary to import the
[`Get`](https://substrate.dev/rustdocs/v3.0.0/frame_support/traits/trait.Get.html) trait from `frame_support`

```rust, ignore
use frame_support::traits::Get;
```

Configurable constants are declared as associated types in the pallet's configuration trait using
the `Get<T>` syntax for any type `T`.

```rust, ignore
pub trait Config: frame_system::Config {
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

	/// Maximum amount added per invocation
	type MaxAddend: Get<u32>;

	/// Frequency with which the stored value is deleted
	type ClearFrequency: Get<Self::BlockNumber>;
}
```

In order to make these constants and their values appear in the runtime metadata, it is necessary to
declare them with the `const` syntax. Usually constants are declared at
the top of this block, right after `fn deposit_event`.

```rust, ignore
#[pallet::config]
pub trait Config: frame_system::Config {
	type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

	/// Maximum amount added per invocation
	type MaxAddend: Get<u32>;

	/// Frequency with which the stored value is deleted
	type ClearFrequency: Get<Self::BlockNumber>;
}
```

This example manipulates a single value in storage declared as `SingleValue`.

```rust, ignore
#[pallet::storage]
#[pallet::getter(fn single_value)]
pub(super) type SingleValue<T: Config> = StorageValue<_, u32, ValueQuery>;
```

`SingleValue` is set to `0` every `ClearFrequency` number of blocks in the `on_finalize` function
that runs at the end of blocks execution.

```rust, ignore
#[pallet::hooks]
impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
	fn on_finalize(n: T::BlockNumber) {
		if (n % T::ClearFrequency::get()).is_zero() {
			let c_val = SingleValue::<T>::get();
			SingleValue::<T>::put(0u32);
			Self::deposit_event(Event::Cleared(c_val));
		}
	}
}
```

Signed transactions may invoke the `add_value` runtime method to increase `SingleValue` as long as
each call adds less than `MaxAddend`. _There is no anti-sybil mechanism so a user could just split a
larger request into multiple smaller requests to overcome the `MaxAddend`_, but overflow is still
handled appropriately.

```rust, ignore
#[pallet::weight(10_000)]
pub fn add_value(origin: OriginFor<T>, val_to_add: u32) -> DispatchResultWithPostInfo {
	let _ = ensure_signed(origin)?;
	ensure!(
		val_to_add <= T::MaxAddend::get(),
		"value must be <= maximum add amount constant"
	);

	// previous value got
	let c_val = SingleValue::<T>::get();

	// checks for overflow when new value added
	let result = match c_val.checked_add(val_to_add) {
		Some(r) => r,
		None => {
			return Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo::from(()),
				error: DispatchError::Other("Addition overflowed"),
			})
		}
	};
	SingleValue::<T>::put(result);
	Self::deposit_event(Event::Added(c_val, val_to_add, result));
	Ok(().into())
}
```

In more complex patterns, the constant value may be used as a static, base value that is scaled by a
multiplier to incorporate stateful context for calculating some dynamic fee (i.e. floating
transaction fees).

## Supplying the Constant Value

When the pallet is included in a runtime, the runtime developer supplies the value of the constant
using the
[`parameter_types!` macro](https://substrate.dev/rustdocs/v3.0.0/frame_support/macro.parameter_types.html). This
pallet is included in the `super-runtime` where we see the following macro invocation and trait
implementation.

```rust
parameter_types! {
	pub const MaxAddend: u32 = 1738;
	pub const ClearFrequency: u32 = 10;
}

#[pallet::config]
pub trait Config: frame_system::Config {
	type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;

	/// Maximum amount added per invocation
	type MaxAddend: Get<u32>;

	/// Frequency with which the stored value is deleted
	type ClearFrequency: Get<Self::BlockNumber>;
}
```
