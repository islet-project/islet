#![no_std]
#![allow(unused_variables)]

extern crate alloc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire,Release};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;
use lazy_static::lazy_static;
use alloc::sync::Arc;
use spin::Mutex;
use alloc::vec::Vec;
use core::marker::Sync;
use core::marker::Sized;
use core::ops::Drop;
use core::marker::Send;
use core::ptr::addr_of;

use armv9a::regs::*;

fn get_pcpu_id() -> usize {
    let (cluster, core) = pid();
    cluster * 4 + core
}

#[inline(always)]
fn pid() -> (usize, usize) {
    unsafe {
        (
            MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF2) as usize,
            MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF1) as usize,
        )
    }
}


struct Edge{
    cid : usize,
    lockset : BTreeSet<usize>,
}

impl Edge{
    pub fn new(cid_ : usize, lock_set : BTreeSet<usize>) -> Self {
        Edge{cid : cid_, lockset : lock_set}
    }
}

struct Edgelist{
    Edges : BTreeMap<(usize,usize), Edge>,
}

impl Edgelist{
    pub fn new() -> Self{
        Edgelist{Edges: BTreeMap::new()}
    }
    
    pub fn addEdge(&mut self, newEdge : Edge, to : usize, from : usize){
        self.Edges.insert((to,from), newEdge);
    }
    
    pub fn check_valid(&mut self, a1: usize, b1: usize, a2: usize, b2 : usize)->bool{
        let Edge1 = self.Edges.get(&(a1,b1)).unwrap();
        let Edge2 = self.Edges.get(&(a2,b2)).unwrap();
        let mut a = true;
        if Edge1.cid == Edge2.cid{
            a = false;
        } 
        
        for ind in Edge1.lockset.clone(){
            for ind2 in Edge2.lockset.clone(){
                if ind == ind2 {
                    a = false;
                    break;
                }
            }
        }
        
        return a;
    }
}

struct Cpulocklist{
    locklist : BTreeMap<usize, BTreeSet<usize>>,
}

impl Cpulocklist{
    pub fn new() -> Self{
        Cpulocklist{locklist : BTreeMap::new()}
    }
    
    pub fn add(&mut self, cid_ : usize, lock_index : usize){
        if !self.locklist.contains_key(&cid_) {
        self.locklist.insert(cid_, BTreeSet::new());
        }

        let temp = self.locklist.get_mut(&cid_).unwrap().clone();
        
        if !temp.contains(&lock_index){
            self.locklist.get_mut(&cid_).unwrap().insert(lock_index);
        }
    }
    
    pub fn get_set(&mut self, cid_ : usize)->BTreeSet<usize>{
        self.locklist.get(&cid_).unwrap().clone()
    }
    
    pub fn remove(&mut self,cid_ : usize, lock_index : usize){
        let temp_set = self.get_set(cid_);
        if temp_set.contains(&lock_index){
            self.locklist.get_mut(&cid_).unwrap().remove(&lock_index);
        }
    }
}

struct Elockgraph{
    adjacency_list : BTreeMap<usize, BTreeSet<usize>>,
}

impl Elockgraph{
    pub fn new() -> Self{
        Elockgraph{adjacency_list: BTreeMap::new()}
    }
    
    pub fn add_Edge(&mut self, from : usize, to : usize){
        if self.adjacency_list.contains_key(&from) && self.adjacency_list.contains_key(&to){
            self.adjacency_list.get_mut(&from).unwrap().insert(to);
        }
    }
    
    pub fn check_deadlock(&self)->bool{

        let Edgelist_temp = Arc::clone(&Edgeslist);
        let mut Edge_ = Edgelist_temp.lock();
        
        for &node in self.adjacency_list.keys(){
            let mut visited = BTreeSet::new();
            let mut path = Vec::new();
            
            if self.dfs(node, node, &mut visited, &mut path){
                let mut val = false;
                for (a,b) in &path{
                    for (c,d) in &path{
                        if !(*a==*c && *b == *d){
                            val = Edge_.check_valid(*a,*b,*c,*d);
                        }
                        else if *a == *b && *b == *c && *c == *d {
                            val = true;
                        }
                    }
                }
                
                if val == true {
                core::mem::drop(Edge_);
                    return val;
                }
            }
        }
        core::mem::drop(Edge_);
        
        return false;
    }
    
    pub fn dfs(&self, start : usize, curr : usize, visited : &mut BTreeSet<usize>, path : &mut Vec<(usize,usize)>)->bool{
        visited.insert(curr);
        if let Some(nodes) = self.adjacency_list.get(&curr){
            for &node in nodes {
                if !visited.contains(&node){
                    path.push((curr,node));
                    if self.dfs(start,node,visited,path){
                        return true;
                    }
                }
                else if node == start {
                    path.push((curr,node));
                    return true;
                }
            }
        }
        
        path.pop();
        visited.remove(&curr);
        return false;
    }
}

lazy_static! {
    static ref elgraph : Arc<Mutex<Elockgraph>> = Arc::new(Mutex::new(Elockgraph::new()));
    static ref tlocklist : Arc<Mutex<Cpulocklist>> = Arc::new(Mutex::new(Cpulocklist::new()));
    static ref Edgeslist : Arc<Mutex<Edgelist>> = Arc::new(Mutex::new(Edgelist::new()));
}

struct RawSpinlock {
    lock : AtomicBool,
}

impl RawSpinlock {

    pub const fn new() -> RawSpinlock {
        RawSpinlock{lock : AtomicBool::new(false)}
    }
    
    pub fn id(&self) -> usize {
        addr_of!(self.lock) as *const _ as usize
    }

    fn lock(&self)-> Result < (), &str > {
          
          #[cfg(feature="deadlock_test")]
          {
          let cid_temp = get_pcpu_id();
          let tlock_temp = Arc::clone(&tlocklist);
          let graph_temp = Arc::clone(&elgraph);
          let Edgelist_temp = Arc::clone(&Edgeslist);
          let mut tlock_ = tlock_temp.lock();
          let mut graph_ = graph_temp.lock();
          let mut Edge_ = Edgelist_temp.lock();
          
          if !graph_.adjacency_list.contains_key(&self.id()){
            graph_.adjacency_list.insert(self.id(), BTreeSet::new());
          }
        
          if tlock_.locklist.contains_key(&cid_temp){
            let BTreeSet_temp = tlock_.get_set(cid_temp);
           
            for ind in BTreeSet_temp.iter() {
                 let index = self.id();
                 graph_.add_Edge(*ind,index);
                 let temp_Edge = Edge ::new(cid_temp,BTreeSet_temp.clone());
                 Edge_.addEdge(temp_Edge,*ind,index);
            }
          }

          core::mem::drop(tlock_);
          core::mem::drop(graph_);
          core::mem::drop(Edge_);
          
        }

        let mut spincount:usize = 0;

        while self.lock.compare_and_swap(false,true,Acquire)!=false {
             spincount+=1;

             #[cfg(feature="deadlock_test")]
             {
             if spincount > 1000000{
                 let graph_temp = Arc::clone(&elgraph);
                 let graph_ = graph_temp.lock();
                 if graph_.check_deadlock() {
                    return Err("DEADLOCK DETECTED, ABORTING THE EXECUTION!!");
                 }
                
                 core::mem::drop(graph_);
             }
            }
        };

        #[cfg(feature="deadlock_test")]
        {
        let tlock_temp = Arc::clone(&tlocklist);
        let mut tlock_ = tlock_temp.lock();
        let ind = self.id();
        tlock_.add(get_pcpu_id(),ind);
        core::mem::drop(tlock_);
        }
        
        #[cfg(feature="p_deadlock_test")]
        {
        let graph_temp = Arc::clone(&elgraph);
        let graph_ = graph_temp.lock();
        if graph_.check_deadlock() {
           return Err("POTENTIAL DEADLOCK DETECTED, ABORTING THE EXECUTION!!");
        }
        core::mem::drop(graph_);
        }

        Ok(())
    }
    

    fn lock_(&self, IID : usize)-> Result < (), &str > {

        #[cfg(feature="deadlock_test")]
        {
        let cid_temp = get_pcpu_id();
        let tlock_temp = Arc::clone(&tlocklist);
        let graph_temp = Arc::clone(&elgraph);
        let Edgelist_temp = Arc::clone(&Edgeslist);
        let mut tlock_ = tlock_temp.lock();
        let mut graph_ = graph_temp.lock();
        let mut Edge_ = Edgelist_temp.lock();
        
        if !graph_.adjacency_list.contains_key(&self.id()){
          graph_.adjacency_list.insert(self.id(), BTreeSet::new());
        }
      
        if tlock_.locklist.contains_key(&IID){
          let BTreeSet_temp = tlock_.get_set(IID);
         
          for ind in BTreeSet_temp.iter() {
               let index = self.id();
               graph_.add_Edge(*ind,index);
               let temp_Edge = Edge ::new(IID,BTreeSet_temp.clone());
               Edge_.addEdge(temp_Edge,*ind,index);
          }
        }

        core::mem::drop(tlock_);
        core::mem::drop(graph_);
        core::mem::drop(Edge_);
        
        }

        let mut spincount:usize = 0;

      
      while self.lock.compare_and_swap(false,true,Acquire)!=false {
           spincount+=1;

        #[cfg(feature="deadlock_test")]
        {
           if spincount > 1000000{
               let graph_temp = Arc::clone(&elgraph);
               let graph_ = graph_temp.lock();
               if graph_.check_deadlock() {
                  return Err("DEADLOCK DETECTED, ABORTING THE EXECUTION!!");
               }
              
               core::mem::drop(graph_);
           }
        }
      };

    #[cfg(feature="deadlock_test")]
    {
      let tlock_temp = Arc::clone(&tlocklist);
      let mut tlock_ = tlock_temp.lock();
      let ind = self.id();
      tlock_.add(IID,ind);
      core::mem::drop(tlock_);
    }
    
    #[cfg(feature="p_deadlock_test")]
    {
    let graph_temp = Arc::clone(&elgraph);
    let graph_ = graph_temp.lock();
    if graph_.check_deadlock() {
       return Err("POTENTIAL DEADLOCK DETECTED, ABORTING THE EXECUTION!!");
    }
    core::mem::drop(graph_);
    }

      Ok(())
    }
    

    fn unlock(&self) {
        #[cfg(feature="deadlock_test")]
        {
         let tlock_temp = Arc::clone(&tlocklist);
         let mut tlock_ = tlock_temp.lock();
         let index = self.id();
         let cpu_id = get_pcpu_id();
         tlock_.remove(cpu_id,index);
         core::mem::drop(tlock_);
        }

        self.lock.store(false,Release);
    }
    
}

pub struct Spinlock<T: ?Sized> {
    lock: RawSpinlock,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Spinlock<T> {}
unsafe impl<T: ?Sized + Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: RawSpinlock::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn findid(&self) -> usize {
        self.lock.id()
    }
    
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Spinlock<T> {

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        let result = self.lock.lock();

        match result {
            Ok(()) => {
                SpinlockGuard {
                    lock: self,
                    _marker: PhantomData,
                }
            }

            Err(e) => {
                panic!("{:?}",e);
            }
        }
    }

    pub fn lock_(&self,IID : usize) -> SpinlockGuard<'_, T> {
        let result = self.lock.lock_(IID);

        match result {
            Ok(()) => {
                SpinlockGuard {
                    lock: self,
                    _marker: PhantomData,
                }
            }

            Err(e) => {
                panic!("{:?}",e);
            }
        }
    }

    pub unsafe fn get(&self) -> &T {
        &*self.data.get()
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

pub struct SpinlockGuard<'a, T: ?Sized> {
    lock: &'a Spinlock<T>,
    _marker: PhantomData<*const ()>,
}

impl<'a, T: ?Sized> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.unlock();
    }
}

impl<'a, T: ?Sized> Deref for SpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

pub fn checkdeadlock()->bool{
    let graph_temp = Arc::clone(&elgraph);
    let graph_ = graph_temp.lock();
    if graph_.check_deadlock() {
        return true;
    }
    return false;
}

pub fn checkdeadlock_weak(node : usize) -> bool {
    let mut visited = BTreeSet::new();
    let mut path = Vec::new();
    let graph_temp = Arc::clone(&elgraph);
    let graph_ = graph_temp.lock();
    let temp = graph_.dfs(node,node,&mut visited,&mut path);
    temp
}
