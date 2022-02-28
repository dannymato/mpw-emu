use crate::common::OSErr;

use super::{EmuState, EmuUC, FuncResult, helpers::{ArgReader, UnicornExtras}};

fn stub_return_void(_uc: &mut EmuUC, _state: &mut EmuState, _reader: &mut ArgReader) -> FuncResult {
	Ok(None)
}

fn new_handle(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let size: u32 = reader.read1(uc)?;
	let handle = state.heap.new_handle(uc, size)?;
	Ok(Some(handle))
}

fn new_ptr(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let size: u32 = reader.read1(uc)?;
	let ptr = state.heap.new_ptr(uc, size)?;
	Ok(Some(ptr))
}

fn dispose_ptr(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let ptr: u32 = reader.read1(uc)?;
	state.heap.dispose_ptr(uc, ptr)?;
	Ok(None)
}

fn get_ptr_size(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let ptr: u32 = reader.read1(uc)?;
	Ok(Some(state.heap.get_ptr_size(uc, ptr)?))
}

fn set_ptr_size(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let (ptr, new_size): (u32, u32) = reader.read2(uc)?;
	state.heap.set_ptr_size(uc, ptr, new_size)?;
	Ok(None)
}

fn dispose_handle(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let handle: u32 = reader.read1(uc)?;
	state.heap.dispose_handle(uc, handle)?;
	Ok(None)
}

fn get_handle_size(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let handle: u32 = reader.read1(uc)?;
	Ok(Some(state.heap.get_handle_size(uc, handle)?))
}

fn set_handle_size(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let (handle, new_size): (u32, u32) = reader.read2(uc)?;
	state.heap.set_handle_size(uc, handle, new_size)?;
	Ok(None)
}

fn block_move_data(uc: &mut EmuUC, _state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let (src, dest, len): (u32, u32, u32) = reader.read3(uc)?;

	if src < dest {
		for i in 0..len {
			uc.write_u8(dest + i, uc.read_u8(src + i)?)?;
		}
	} else if src > dest {
		for i in 0..len {
			let inv_i = len - 1 - i;
			uc.write_u8(dest + inv_i, uc.read_u8(src + inv_i)?)?;
		}
	}

	Ok(None)
}

fn h_get_state(_uc: &mut EmuUC, _state: &mut EmuState, _reader: &mut ArgReader) -> FuncResult {
	// We don't implement this
	Ok(Some(0))
}

fn ptr_and_hand(uc: &mut EmuUC, state: &mut EmuState, reader: &mut ArgReader) -> FuncResult {
	let (ptr, handle, size): (u32, u32, u32) = reader.read3(uc)?;

	let current_size = state.heap.get_handle_size(uc, handle)?;
	if state.heap.set_handle_size(uc, handle, current_size + size)? {
		let dest = uc.read_u32(handle)? + current_size;
		for i in 0..size {
			uc.write_u8(dest + i, uc.read_u8(ptr + i)?)?;
		}
		Ok(Some(0))
	} else {
		Ok(Some(OSErr::NotEnoughMemory.to_u32()))
	}
}

pub(super) fn install_shims(state: &mut EmuState) {
	state.install_shim_function("NewHandle", new_handle);
	state.install_shim_function("NewHandleClear", new_handle);
	state.install_shim_function("NewPtr", new_ptr);
	state.install_shim_function("NewPtrClear", new_ptr);
	state.install_shim_function("HLock", stub_return_void);
	state.install_shim_function("HUnlock", stub_return_void);
	state.install_shim_function("HLockHi", stub_return_void);
	state.install_shim_function("MoveHHi", stub_return_void);
	state.install_shim_function("DisposePtr", dispose_ptr);
	state.install_shim_function("GetPtrSize", get_ptr_size);
	state.install_shim_function("SetPtrSize", set_ptr_size);
	state.install_shim_function("DisposeHandle", dispose_handle);
	state.install_shim_function("GetHandleSize", get_handle_size);
	state.install_shim_function("SetHandleSize", set_handle_size);
	state.install_shim_function("BlockMoveData", block_move_data);
	state.install_shim_function("HGetState", h_get_state);
	state.install_shim_function("HSetState", stub_return_void);
	state.install_shim_function("PtrAndHand", ptr_and_hand);
}
