use utrace_common::{Record, UnsafeKind};

use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::{self, FnKind, Visitor};
use rustc_hir::BlockCheckMode::UnsafeBlock;
use rustc_hir::{
    Block, BodyId, FnDecl, ImplItem, Item, ItemKind, TraitFn, TraitItem, UnsafeSource, Unsafety,
};
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id;
use rustc_span::Span;

pub struct Parser<'tcx> {
    tcx: TyCtxt<'tcx>,
    record: Record,
}

impl<'tcx> Parser<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let krate = tcx.crate_name(def_id::LOCAL_CRATE).to_string();

        Self {
            tcx,
            record: Record::new(krate),
        }
    }

    pub fn run(&mut self) {
        self.tcx.hir().visit_all_item_likes_in_crate(self);
    }

    pub fn save(&self) {
        self.record.save(&utrace_common::config::out_dir()).unwrap();
    }
}

impl<'tcx> Visitor<'tcx> for Parser<'tcx> {
    fn visit_block(&mut self, block: &'tcx Block<'tcx>) {
        if block.rules == UnsafeBlock(UnsafeSource::UserProvided) {
            let owner_id = self.tcx.hir().get_parent_item(block.hir_id);
            let def_path = self.tcx.def_path(owner_id.into());
            let mut fn_name = def_path.to_string_no_crate_verbose();
            if fn_name.contains("impl") {
                fn_name = format!("::{}", self.tcx.def_path_str(owner_id));
            };
            self.record.add(UnsafeKind::Block, fn_name);
        }
        intravisit::walk_block(self, block);
    }

    fn visit_impl_item(&mut self, item: &'tcx ImplItem<'tcx>) {
        if let rustc_hir::ImplItemKind::Fn(_, body_id) = &item.kind {
            let body = self.tcx.hir().body(*body_id);
            self.visit_body(body);
        }
        intravisit::walk_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'tcx TraitItem<'tcx>) {
        if let rustc_hir::TraitItemKind::Fn(fn_sig, trait_fn) = &item.kind {
            if let TraitFn::Provided(body_id) = trait_fn {
                let body = self.tcx.hir().body(*body_id);
                self.visit_body(body);
            }

            if let TraitFn::Required(_) = trait_fn {
                if fn_sig.header.unsafety == Unsafety::Unsafe {
                    let def_path = self.tcx.def_path(item.owner_id.to_def_id());
                    let fn_name = def_path.to_string_no_crate_verbose();
                    self.record.add(UnsafeKind::Function, fn_name);
                }
            }
        }
        intravisit::walk_trait_item(self, item);
    }

    fn visit_fn(
        &mut self,
        fk: FnKind<'tcx>,
        fd: &'tcx FnDecl<'tcx>,
        b: BodyId,
        _: Span,
        id: LocalDefId,
    ) {
        let header = match fk {
            intravisit::FnKind::ItemFn(_, _, header) => header,
            intravisit::FnKind::Method(_, sig) => sig.header,
            _ => return,
        };

        if header.unsafety == Unsafety::Unsafe {
            let def_path = self.tcx.def_path(id.to_def_id());
            let mut fn_name = def_path.to_string_no_crate_verbose();
            if fn_name.contains("impl") {
                fn_name = format!("::{}", self.tcx.def_path_str(id));
            };
            self.record.add(UnsafeKind::Function, fn_name);
        }

        intravisit::walk_fn(self, fk, fd, b, id);
    }

    fn visit_item(&mut self, item: &'tcx Item<'tcx>) {
        if let ItemKind::Fn(_, _, body_id) = &item.kind {
            let body = self.tcx.hir().body(*body_id);
            self.visit_body(body);
        }

        if let ItemKind::Trait(_, unsafety, _, _, _) = &item.kind {
            if *unsafety == Unsafety::Unsafe {
                let def_path = self.tcx.def_path(item.owner_id.to_def_id());
                let trait_name = def_path.to_string_no_crate_verbose();
                self.record.add(UnsafeKind::Trait, trait_name);
            }
        }

        if let ItemKind::Impl(ref_) = &item.kind {
            if ref_.unsafety == Unsafety::Unsafe {
                let impl_name = format!("::{}", self.tcx.def_path_str(item.owner_id));
                self.record.add(UnsafeKind::Impl, impl_name);
            }
        }

        intravisit::walk_item(self, item);
    }
}
