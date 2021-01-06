use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Not};

use hibitset::{BitSet, BitSetNot};

use tb_core::Id;
use tb_storage::{Storage, StorageItems};

use crate::entity::Entities;
use crate::join::Join;
use crate::system::data::{access_order, AccessOrder};
use crate::world::ResourceId;
use crate::{Entity, SystemData, World};

pub trait Component: 'static + Sized + Clone {
    type StorageItems: StorageItems<Data = Self>;
}

pub trait EntityRef {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity));
}

pub trait ComponentWithEntityRef<'e>: Component {
    type Ref: 'e + EntityRef;
    fn get_entity_ref(&'e mut self) -> Self::Ref;
}

pub struct ComponentStorage<C: Component> {
    storage: Storage<C::StorageItems>,
    _phantom: PhantomData<C>,
}

pub struct Components<'r, S: 'r, C: Component, A: AccessOrder> {
    entities: &'r Entities,
    storage: S,
    _phantom: PhantomData<(C, A)>,
}

pub type ReadComponents<'r, C, A> = Components<'r, &'r ComponentStorage<C>, C, A>;
pub type RBWComponents<'r, C> = ReadComponents<'r, C, access_order::ReadBeforeWrite>;
pub type RAWComponents<'r, C> = ReadComponents<'r, C, access_order::ReadAfterWrite>;
pub type WriteComponents<'r, C> =
    Components<'r, &'r mut ComponentStorage<C>, C, access_order::Write>;

pub struct AntiComponents<'r> {
    mask: BitSetNot<&'r BitSet>,
}

impl<'e> EntityRef for &'e mut Entity {
    fn for_each(&mut self, action: &mut impl FnMut(&mut Entity)) {
        action(self)
    }
}

macro_rules! impl_entity_ref_tuple {
    ($e:ident) => {};
    ($e0:ident, $($e1:ident), +) => {
        impl_entity_ref_tuple!($($e1), +);

        impl<'e, $e0: EntityRef, $($e1: EntityRef), +> EntityRef for ($e0, $($e1), +) {

            #[allow(non_snake_case)]
            fn for_each(&mut self, action: &mut impl FnMut(&mut Entity)) {
                let ($e0, $($e1), +) = self;
                $e0.for_each(action);
                $($e1.for_each(action));
                +;
            }
        }
    };
}

impl_entity_ref_tuple!(E0, E1, E2, E3, E4, E5, E6, E7);

impl<C: Component> Default for ComponentStorage<C> {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<C: Component> Deref for ComponentStorage<C> {
    type Target = Storage<C::StorageItems>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<C: Component> DerefMut for ComponentStorage<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

impl<'r, C: Component, A: AccessOrder> Not for &'r ReadComponents<'r, C, A> {
    type Output = AntiComponents<'r>;

    fn not(self) -> Self::Output {
        AntiComponents::new(self.storage.open().0)
    }
}

impl<'r, C: Component> Not for &'r mut WriteComponents<'r, C> {
    type Output = AntiComponents<'r>;

    fn not(self) -> Self::Output {
        AntiComponents::new(self.storage.open().0)
    }
}

impl<'r> AntiComponents<'r> {
    fn new(mask: &'r BitSet) -> Self {
        Self {
            mask: BitSetNot(mask),
        }
    }
}

impl<'r> Join for AntiComponents<'r> {
    type BitSet = BitSetNot<&'r BitSet>;
    type Component = ();
    type Components = ();

    fn open(self) -> (Self::BitSet, Self::Components) {
        (self.mask, ())
    }

    unsafe fn get(_components: &mut Self::Components, _id: Id) -> Self::Component {}
}

impl<'r, C: Component, A: AccessOrder> Join for &'r ReadComponents<'r, C, A> {
    type BitSet = &'r BitSet;
    type Component = &'r C;
    type Components = &'r C::StorageItems;

    fn open(self) -> (Self::BitSet, Self::Components) {
        self.storage.open()
    }

    unsafe fn get(components: &mut Self::Components, id: Id) -> Self::Component {
        components.get(id)
    }
}

impl<'r, C: Component> Join for &'r mut WriteComponents<'r, C> {
    type BitSet = &'r BitSet;
    type Component = &'r mut C;
    type Components = &'r mut C::StorageItems;

    fn open(self) -> (Self::BitSet, Self::Components) {
        self.storage.open_mut()
    }

    unsafe fn get(components: &mut Self::Components, id: Id) -> Self::Component {
        let components: *mut Self::Components = components as *mut Self::Components;
        (*components).get_mut(id)
    }
}

impl<'r, C: Component, A: AccessOrder> Deref for ReadComponents<'r, C, A> {
    type Target = ComponentStorage<C>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<'r, C: Component> Deref for WriteComponents<'r, C> {
    type Target = ComponentStorage<C>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<'r, C: Component> DerefMut for WriteComponents<'r, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

impl<'r, C: Component, A: AccessOrder> ReadComponents<'r, C, A> {
    fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_storage::<C>(),
            _phantom: Default::default(),
        }
    }
}

impl<'r, C: Component> WriteComponents<'r, C> {
    fn new(world: &'r World) -> Self {
        Self {
            entities: world.fetch(),
            storage: world.fetch_storage_mut::<C>(),
            _phantom: Default::default(),
        }
    }
}

impl<'r, C: Component> SystemData<'r> for RBWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn reads_before_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::StorageItems>(),
        ]
    }
}

impl<'r, C: Component> SystemData<'r> for WriteComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<C::StorageItems>()]
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![ResourceId::new::<Entities>()]
    }
}

impl<'r, C: Component> SystemData<'r> for RAWComponents<'r, C> {
    fn fetch(world: &'r World) -> Self {
        Self::new(world)
    }

    fn reads_after_write() -> Vec<ResourceId> {
        vec![
            ResourceId::new::<Entities>(),
            ResourceId::new::<C::StorageItems>(),
        ]
    }
}

impl World {
    pub fn fetch_storage<C: Component>(&self) -> &ComponentStorage<C> {
        self.fetch()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn fetch_storage_mut<C: Component>(&self) -> &mut ComponentStorage<C> {
        self.fetch_mut()
    }

    pub fn insert_storage<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.insert(ComponentStorage::<C>::default)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[component(VecStorageItems)]
    struct Component1 {
        value1: i32,
    }

    #[component]
    struct Component2 {
        value2: i32,
    }

    #[test]
    fn it_works() {
        let mut world = World::default();
        world.insert(Entities::default);
        world.insert(ComponentStorage::<Component1>::default);
        world.insert(ComponentStorage::<Component2>::default);
        let components1 = RAWComponents::<Component1>::fetch(&world);
        let mut components2 = WriteComponents::<Component2>::fetch(&world);
        for _x in (&components1, &mut components2).join() {
            unreachable!()
        }

        let _entity = world
            .create_entity()
            .with(Component1 { value1: 1 })
            .with(Component2 { value2: 2 })
            .create();
        let components1 = RAWComponents::<Component1>::fetch(&world);
        let mut components2 = WriteComponents::<Component2>::fetch(&world);
        let (v1, v2): (&Component1, &mut Component2) =
            (&components1, &mut components2).join().next().unwrap();
        assert_eq!(v1.value1, 1);
        assert_eq!(v2.value2, 2);
    }

    #[test]
    fn anti_components() {
        let mut world = World::default();
        world
            .create_entity()
            .with(Component1 { value1: 1 })
            .with(Component2 { value2: 2 })
            .create();
        world
            .create_entity()
            .with(Component1 { value1: 11 })
            .create();

        let (components1, components2) =
            <(RBWComponents<Component1>, RBWComponents<Component2>)>::fetch(&world);
        let mut has = false;
        for (component1, component2) in (&components1, &components2).join() {
            has = true;
            assert_eq!(component1.value1, 1);
            assert_eq!(component2.value2, 2);
        }
        assert!(has);

        let mut has = false;
        for (component1, _) in (&components1, !&components2).join() {
            has = true;
            assert_eq!(component1.value1, 11);
        }
        assert!(has);

        for (_, _) in (!&components1, &components2).join() {
            unreachable!()
        }
    }
}
