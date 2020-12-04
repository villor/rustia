use shipyard::*;

pub struct Parent {
    pub num_children: usize,
    pub first_child: EntityId,
}

pub struct Child {
    pub parent: EntityId,
    pub prev: EntityId,
    pub next: EntityId,
}

//pub type HierarchyViewMut<'a> = (EntitiesViewMut<'a>, ViewMut<'a, Parent>, ViewMut<'a, Child>);

pub trait Hierarchy {
    fn detach(&mut self, id: EntityId);
    fn attach(&mut self, id: EntityId, parent: EntityId);
    fn attach_new(&mut self, parent: EntityId) -> EntityId;
}

impl Hierarchy for (EntitiesViewMut<'_>, ViewMut<'_, Parent>, ViewMut<'_, Child>) {
    fn detach(&mut self, id: EntityId) {
        let (_, parents, children) = self;
        // remove the Child component - if nonexistent, do nothing
        if let Some(child) = children.remove(id) {
            // retrieve and update Parent component from ancestor
            let parent = &mut parents[child.parent];
            parent.num_children -= 1;
    
            if parent.num_children == 0 {
                // if the number of children is zero, the Parent component must be removed
                parents.remove(child.parent);
            } else {
                // the ancestor still has children, and we have to change some linking
                // check if we have to change first_child
                if parent.first_child == id {
                    parent.first_child = child.next;
                }
                // remove the detached child from the sibling chain
                children[child.prev].next = child.next;
                children[child.next].prev = child.prev;
            }
        }
    }

    fn attach(&mut self, id: EntityId, parent: EntityId) {
        // the entity we want to attach might already be attached to another parent
        self.detach(id);
    
        let (entities, parents, children) = self;
    
        // either the designated parent already has a Parent component – and thus one or more children
        if let Ok(p) = parents.try_get(parent) {
            // increase the parent's children counter
            p.num_children += 1;
    
            // get the ids of the new previous and next siblings of our new child
            let prev = children[p.first_child].prev;
            let next = p.first_child;
    
            // change the linking
            children[prev].next = id;
            children[next].prev = id;
            
            // add the Child component to the new entity
            entities.add_component(children, Child { parent, prev, next }, id);
        } else {
            // in this case our designated parent is missing a Parent component
            // we don't need to change any links, just insert both components
            entities.add_component(
                children,
                Child {
                    parent,
                    prev: id,
                    next: id,
                },
                id,
            );
            entities.add_component(
                parents,
                Parent {
                    num_children: 1,
                    first_child: id,
                },
                parent,
            );
        }
    }

    fn attach_new(&mut self, parent: EntityId) -> EntityId {
        let id = self.0.add_entity((), ());
        self.attach(id, parent);
        id
    }
}

pub struct ChildrenIter<C> {
    get_child: C,
    cursor: (EntityId, usize),
}

impl<'a, C> Iterator for ChildrenIter<C>
where
    C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
            let ret = self.cursor.0;
            self.cursor.0 = self.get_child.get(self.cursor.0).next;
            Some(ret)
        } else {
            None
        }
    }
}

pub struct AncestorIter<C> {
    get_child: C,
    cursor: EntityId,
}

impl<'a, C> Iterator for AncestorIter<C>
where
    C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_child.try_get(self.cursor).ok().map(|child| {
            self.cursor = child.parent;
            child.parent
        })
    }
}

pub struct DescendantsIter<P, C> {
    get_parent: P,
    get_child: C,
    cursors: Vec<(EntityId, usize)>,
}

impl<'a, P, C> Iterator for DescendantsIter<P, C>
where
    P: Get<Out = &'a Parent> + Copy,
    C: Get<Out = &'a Child> + Copy,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cursor) = self.cursors.last_mut() {
            if cursor.1 > 0 {
                cursor.1 -= 1;
                let ret = cursor.0;
                cursor.0 = self.get_child.get(cursor.0).next;
                if let Ok(parent) = self.get_parent.try_get(ret) {
                    self.cursors.push((parent.first_child, parent.num_children));
                }
                Some(ret)
            } else {
                self.cursors.pop();
                self.next()
            }
        } else {
            None
        }
    }
}

pub trait HierarchyIter<'a, P, C> {
    fn ancestors(&self, id: EntityId) -> AncestorIter<C>;
    fn children(&self, id: EntityId) -> ChildrenIter<C>;
    fn descendants(&self, id: EntityId) -> DescendantsIter<P, C>;
}

impl<'a, P, C> HierarchyIter<'a, P, C> for (P, C)
where
    P: Get<Out = &'a Parent> + Copy,
    C: Get<Out = &'a Child> + Copy,
{
    fn ancestors(&self, id: EntityId) -> AncestorIter<C> {
        let (_, children) = self;

        AncestorIter {
            get_child: *children,
            cursor: id,
        }
    }

    fn children(&self, id: EntityId) -> ChildrenIter<C> {
        let (parents, children) = self;

        ChildrenIter {
            get_child: *children,
            cursor: parents
                .try_get(id)
                .map_or((id, 0), |parent| (parent.first_child, parent.num_children)),
        }
    }

    fn descendants(&self, id: EntityId) -> DescendantsIter<P, C> {
        let (parents, children) = self;
        
        DescendantsIter {
            get_parent: *parents,
            get_child: *children,
            cursors: parents.try_get(id).map_or_else(
                |_| Vec::new(),
                |parent| vec![(parent.first_child, parent.num_children)],
            ),
        }
    }
}
