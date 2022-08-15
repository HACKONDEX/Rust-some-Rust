#![forbid(unsafe_code)]
use crate::{
    data::ObjectId,
    error::{Error, NotFoundError, Result},
    object::{Object, Store},
    storage::StorageTransaction,
};
use std::{
    any::{Any, TypeId},
    cell::{Cell, Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

////////////////////////////////////////////////////////////////////////////////

struct MemoryObject {
    id: ObjectId,
    state: Cell<ObjectState>,
    object: RefCell<Box<dyn Store>>,
}

impl MemoryObject {
    pub fn new(id: ObjectId, state: ObjectState, ptr: Box<dyn Store>) -> Self {
        Self {
            id,
            state: Cell::new(state),
            object: RefCell::new(ptr),
        }
    }

    pub fn get_state(&self) -> ObjectState {
        self.state.get()
    }
}

pub struct Transaction<'a> {
    inner: Box<dyn StorageTransaction + 'a>,
    map: RefCell<HashMap<(TypeId, ObjectId), Rc<MemoryObject>>>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self {
            inner,
            map: RefCell::default(),
        }
    }

    fn ensure_table<T: Object>(&self) -> Result<()> {
        if !self.inner.table_exists(T::SCHEMA.table_name)? {
            self.inner.create_table(T::SCHEMA)?;
        }
        Ok(())
    }

    pub fn create<T: Object>(&self, src_obj: T) -> Result<Tx<'_, T>> {
        self.ensure_table::<T>()?;
        let object_id = self
            .inner
            .insert_row(T::SCHEMA, &src_obj.get_row_from_object())?;

        let memory_object = Rc::new(MemoryObject::new(
            object_id,
            ObjectState::Clean,
            Box::new(src_obj),
        ));

        self.map
            .borrow_mut()
            .insert((TypeId::of::<T>(), object_id), memory_object.clone());

        Ok(Tx::new(PhantomData, memory_object))
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        self.ensure_table::<T>()?;

        if let Some(object) = self.map.borrow().get(&(TypeId::of::<T>(), id)).cloned() {
            match object.as_ref().state.get() {
                ObjectState::Removed => {
                    return Err(Error::NotFound(Box::new(NotFoundError::new(
                        id,
                        T::SCHEMA.type_name,
                    ))));
                }
                _ => {
                    return Ok(Tx::new(PhantomData, object));
                }
            }
        }

        let memory_object = Rc::new(MemoryObject::new(
            id,
            ObjectState::Clean,
            Box::new(T::get_object_from_row(
                self.inner.select_row(id, T::SCHEMA)?,
            )),
        ));
        self.map
            .borrow_mut()
            .insert((TypeId::of::<T>(), id), memory_object.clone());

        Ok(Tx::new(PhantomData, memory_object))
    }

    fn try_apply(&self) -> Result<()> {
        for memory_object in self.map.borrow().values() {
            let object = memory_object.object.borrow();
            match memory_object.state.get() {
                ObjectState::Clean => {}
                ObjectState::Removed => {
                    self.inner
                        .delete_row(memory_object.id, object.get_schema())?;
                }
                ObjectState::Modified => {
                    self.inner.update_row(
                        memory_object.id,
                        object.get_schema(),
                        &object.get_row_from_store(),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        self.try_apply()?;
        self.inner.commit()
    }

    pub fn rollback(self) -> Result<()> {
        self.inner.rollback()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

#[derive(Clone)]
pub struct Tx<'a, T> {
    lifetime: PhantomData<&'a T>,
    object: Rc<MemoryObject>,
}

impl<'a, T: Any> Tx<'a, T> {
    fn new(lifetime: PhantomData<&'a T>, object: Rc<MemoryObject>) -> Self {
        Self { lifetime, object }
    }

    pub fn id(&self) -> ObjectId {
        self.object.id
    }

    pub fn state(&self) -> ObjectState {
        self.object.get_state()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        match self.object.get_state() {
            ObjectState::Removed => {
                panic!("cannot borrow a removed object");
            }
            _ => Ref::map(self.object.object.borrow(), |store| {
                store.cast_to_any().downcast_ref().unwrap()
            }),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        match self.object.get_state() {
            ObjectState::Removed => {
                panic!("cannot borrow a removed object");
            }
            _ => {
                self.object.state.set(ObjectState::Modified);
                RefMut::map(self.object.object.borrow_mut(), |x| {
                    x.cast_to_any_mut().downcast_mut().unwrap()
                })
            }
        }
    }

    pub fn delete(self) {
        if self.object.object.try_borrow_mut().is_err() {
            panic!("cannot delete a borrowed object");
        }
        self.object.state.set(ObjectState::Removed);
    }
}
