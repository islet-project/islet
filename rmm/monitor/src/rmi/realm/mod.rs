pub(crate) mod params;

use self::params::Params;
use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::REALM_CREATE, |ctx, rmi, _| {
        let addr = ctx.arg[1];
        let param = unsafe { Params::parse(addr) };
        trace!("{:?}", param);

        // TODO:
        //   Validate params
        //   Manage granule including locking
        //   Manage vmid
        //   Keep params in Realm

        let ret = rmi.create_realm();
        match ret {
            Ok(id) => {
                ctx.ret[0] = rmi::SUCCESS;
                ctx.ret[1] = id;
            }
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REC_AUX_COUNT, |ctx, _, _| {
        ctx.ret[0] = rmi::SUCCESS;
        ctx.ret[1] = rmi::MAX_REC_AUX_GRANULES;
    });

    listen!(mainloop, rmi::VCPU_CREATE, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let ret = rmi.create_vcpu(id);
        match ret {
            Ok(vcpuid) => {
                ctx.ret[0] = rmi::RET_SUCCESS;
                ctx.ret[1] = vcpuid;
            }
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REALM_DESTROY, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let ret = rmi.remove(id);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::RET_SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REALM_RUN, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let vcpu = ctx.arg[1];
        let incr_pc = ctx.arg[2];
        let ret = rmi.run(id, vcpu, incr_pc);
        match ret {
            Ok(val) => match val[0] {
                rmi::RET_EXCEPTION_TRAP | rmi::RET_EXCEPTION_IRQ => {
                    ctx.ret = [val[0], val[1], val[2], val[3], 0, 0, 0, 0];
                }
                _ => ctx.ret[0] = rmi::RET_SUCCESS,
            },
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        };
    });

    listen!(mainloop, rmi::REALM_MAP_MEMORY, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let guest = ctx.arg[1];
        let phys = ctx.arg[2];
        let size = ctx.arg[3];
        // prot: bits[0] : writable, bits[1] : fault on exec, bits[2] : device
        let prot = ctx.arg[4]; // bits[3]
        let ret = rmi.map(id, guest, phys, size, prot);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::RET_SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REALM_UNMAP_MEMORY, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let guest = ctx.arg[1];
        let size = ctx.arg[2];
        let ret = rmi.unmap(id, guest, size);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::RET_SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REALM_SET_REG, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let vcpu = ctx.arg[1];
        let register = ctx.arg[2];
        let value = ctx.arg[3];
        let ret = rmi.set_reg(id, vcpu, register, value);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::RET_SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });

    listen!(mainloop, rmi::REALM_GET_REG, |ctx, rmi, _| {
        let id = ctx.arg[0];
        let vcpu = ctx.arg[1];
        let register = ctx.arg[2];
        let ret = rmi.get_reg(id, vcpu, register);
        match ret {
            Ok(_) => ctx.ret[0] = rmi::RET_SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }
    });
}
