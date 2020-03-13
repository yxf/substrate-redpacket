
# Build a blockchain RedPacket with Substrate

#### Introduction
RedPacket is a easy way to airdrop. Anyone can claim some funds from a valid RedPacket that was created by someone. 

#### Data structure and storage items

```rust
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Packet<PacketId, Balance, BlockNumber, AccountId> {
    id: PacketId, // Unique id
    total: Balance, // Total funds supplied
    unclaimed: Balance,
    count: u32, // How many times can be claimed.
    expires_at: BlockNumber, // Expire duration
    owner: AccountId,
    distributed: bool,
}
```

```rust
decl_storage! {
    trait Store for Module<T: Trait> as RedPacket {
        /// Store all created packets
        pub Packets get(fn packets): map T::PacketId => Packet<T::PacketId, BalanceOf<T>, T::BlockNumber, T::AccountId>;
        /// Store all claim records
        pub Claims get(fn claims_of): map T::PacketId => Vec<T::AccountId>;
        /// Increase after created a Packet
        pub NextPacketId get(next_packet_id): T::PacketId;
    }
}
```

#### Dispatchable functions 

![diagram](./seq.png)

- `create` - Create a new RedPacket and reserve creator's funds.
- `claim` - Store a claim record for distribution.
- `distribute` - Unreserve creator's funds and do transfers.



#### TODOs

- Random Redpacket - Upgrade RedPacket to support random claim funds. 

- Auto distribution - Try to do automatic distribution in function `on_finalize` for distributable Redpackets.


## Best Practices

#### Use safe arithmetic functions
There were many attacks on Ethereum Smart Contract because of type overflow. Overflow problem is offen omited by developers, and is very easy attacked. It is very necessary to use safe arithmetic functions when we do arithmetic operations. Substrate provided `trait Saturating`, we can use `saturating_add`, `saturating_sub` and `saturating_mul`. 

For example, We use `saturating_mul` in `RedPacket::create` function when caculating reserved total balances.

```rust
pub fn create(origin, quota: BalanceOf<T>, count: u32, expires: T::BlockNumber) -> DispatchResult {
    /* --snip-- */
    let total = quota.saturating_mul(<BalanceOf<T>>::from(count));
    /* --snip-- */
}
```

#### Check first then update
In Substrate module's function, updating storage operations still be successful before location of the error raised. That's why we must check our logic first before updating. 

Substrate provided `ensure` macro and `ensure_signed` to do checks:

1. `ensure` - The `ensure` macro expects a condition and returns an `Err` if the condition gets false, then the function exits.

2. `ensure_signed`: You should always use `ensure_signed` first in your function to check the call is permitted, otherwise your chain might be attackable.

For example:

```rust
pub fn create(origin, quota: BalanceOf<T>, count: u32, expires: T::BlockNumber) -> DispatchResult {
    // Do checks first
    ensure!(count > 0, Error::<T>::GreaterThanZero);
    ensure!(quota > Zero::zero(), Error::<T>::GreaterThanZero);
    ensure!(expires > Zero::zero(), Error::<T>::GreaterThanZero);
    let sender = ensure_signed(origin)?;

    /* --snip-- */

    // Update finally
    <Packets<T>>::insert(id, new_packet);
    <NextPacketId<T>>::mutate(|id| *id += One::one());
    /* --snip-- */
}
```


#### Use `decl_error!`

Use `decl_error!` to define errors instead of string errors. It keeps code simple and makes errors easy to manage.

Good:

```rust
decl_error! {
    /// Error
    pub enum Error for Module<T: Trait> {
        /// Sender's balance is too low.
        InsufficientBalance,
        /*--snip--*/
    }
}
```
```rust
pub fn create(origin, quota: BalanceOf<T>, count: u32, expires: T::BlockNumber) -> DispatchResult {
    /* --snip--*/
    ensure!(sender_balance >= total, Error::<T>::InsufficientBalance);
    /* --snip--*/
}
```

Bad:

```rust
pub fn create(origin, quota: BalanceOf<T>, count: u32, expires: T::BlockNumber) -> DispatchResult {
    /* --snip--*/
    ensure!(sender_balance >= total, "Insufficient balance");
    /* --snip--*/
}
```

#### Write more tests
Testing is very important, especially in blockchain project. Writing test code guarantees code quality and makes your code easy to read.

