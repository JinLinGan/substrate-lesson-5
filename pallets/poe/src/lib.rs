#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::StaticLookup;

	// The struct on which we build all of our Pallet logic.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/* Placeholder for defining custom types. */

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// For constraining the maximum bytes of a hash used for any proof
		type MaxBytesInHash: Get<u32>;
	}

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 定义创建Claim事件 [who, claim]
		ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		/// 定义Revoke事件 [who, claim]
		ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
		// 定义转移Claim事件
		ClaimTransferred(T::AccountId, T::AccountId, BoundedVec<u8, T::MaxBytesInHash>),
	}

	#[pallet::error]
	pub enum Error<T> {
		// 存证已经存在
		ProofAlreadyClaimed,
		// 存证不存在
		NoSuchProof,
		// 不是存证所有者
		NotProofOwner,
	}

	#[pallet::storage]
	// 定义存储结构
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxBytesInHash>,
		(T::AccountId, T::BlockNumber),
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 定义create_claim 事件
		#[pallet::weight(1_000)]
		pub fn create_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>,
		) -> DispatchResult {
			//验证签名并获取发送者
			let sender = ensure_signed(origin)?;

			// 确认当前存证不存在
			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

			// 获取当前区块
			let current_block = <frame_system::Pallet<T>>::block_number();

			// 保存数据
			Proofs::<T>::insert(&proof, (&sender, current_block));

			// 发送ClaimCreated事件
			Self::deposit_event(Event::ClaimCreated(sender, proof));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>,
		) -> DispatchResult {
			//验证签名并获取发送者
			let sender = ensure_signed(origin)?;

			// 验证存证是否存在
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

			// 获取存证所有者
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

			// 判断存证所有者是不是发送签名的人
			ensure!(sender == owner, Error::<T>::NotProofOwner);

			// 删除存证
			Proofs::<T>::remove(&proof);

			// 发送事件
			Self::deposit_event(Event::ClaimRevoked(sender, proof));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			proof: BoundedVec<u8, T::MaxBytesInHash>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			// 获取目标地址
			let dest = T::Lookup::lookup(dest)?;

			//验证签名并获取发送者
			let sender = ensure_signed(origin)?;

			// 验证存证是否存在
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

			// 获取存证所有者
			let (owner, _) = Proofs::<T>::get(&proof).expect("All proofs must have an owner!");

			// 判断存证所有者是不是发送签名的人
			ensure!(sender == owner, Error::<T>::NotProofOwner);

			// Proofs::<T>::remove(&proof);

			// 获取当前
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Proofs::<T>::insert(&proof, (&dest, current_block));
			// 修改存储
			Proofs::<T>::mutate(&proof, |v| match v {
				None => {},
				Some(_) => *v = Some((dest.clone(), current_block)),
			});

			// 发送转移成功事件
			Self::deposit_event(Event::ClaimTransferred(sender, dest, proof));
			Ok(())
		}
	}
}
