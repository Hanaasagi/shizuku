use llvm_sys::analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction;
use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::LLVMCodeGenFileType::*;
use llvm_sys::target_machine::*;
use std::ffi::CString;
use std::ptr;

fn main() {
    unsafe {
        // Initialize LLVM components
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();

        // Create a new LLVM context and module
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(c_str("hello_module").as_ptr(), context);

        // Create the function signature
        let func_type = LLVMFunctionType(LLVMVoidTypeInContext(context), ptr::null_mut(), 0, 0);
        let func = LLVMAddFunction(module, c_str("main").as_ptr(), func_type);

        // Create a basic block and builder
        let entry = LLVMAppendBasicBlockInContext(context, func, c_str("entry").as_ptr());
        let builder = LLVMCreateBuilderInContext(context);
        LLVMPositionBuilderAtEnd(builder, entry);

        // Create a call to printf
        let printf_type = LLVMFunctionType(
            LLVMInt32TypeInContext(context),
            [LLVMPointerType(LLVMInt8TypeInContext(context), 0)].as_mut_ptr(),
            1,
            1,
        );
        let printf_func = LLVMAddFunction(module, c_str("printf").as_ptr(), printf_type);

        // Create the format string
        let hello_str = c_str("Hello, World!\n");
        let hello_global =
            LLVMBuildGlobalStringPtr(builder, hello_str.as_ptr(), c_str("hello_str").as_ptr());

        // Call printf with the format string
        LLVMBuildCall2(
            builder,
            printf_type,
            printf_func,
            [hello_global].as_mut_ptr(),
            1,
            c_str("").as_ptr(),
        );

        // Return void
        LLVMBuildRetVoid(builder);

        // Verify the module
        if LLVMVerifyModule(module, LLVMAbortProcessAction, ptr::null_mut()) != 0 {
            panic!("Module verification failed");
        }

        // Save the module to a .ll file
        save_module_to_ll(module, "output.ll");

        // Generate assembly from the module
        generate_assembly(module, "output.s");

        // JIT compile and execute
        let mut engine: LLVMExecutionEngineRef = ptr::null_mut();
        let mut error: *mut i8 = ptr::null_mut();
        if LLVMCreateJITCompilerForModule(&mut engine, module, 0, &mut error) != 0 {
            panic!("Failed to create JIT compiler: {}", c_str_from_ptr(error));
        }

        let main_func = LLVMGetNamedFunction(module, c_str("main").as_ptr());
        LLVMRunFunction(engine, main_func, 0, ptr::null_mut());

        // Clean up
        LLVMDisposeBuilder(builder);
        LLVMDisposeExecutionEngine(engine);
        LLVMContextDispose(context);
    }
}

/// Save the LLVM module to a `.ll` file.
unsafe fn save_module_to_ll(module: LLVMModuleRef, filename: &str) { unsafe {
    let c_filename = c_str(filename);
    if LLVMPrintModuleToFile(module, c_filename.as_ptr(), ptr::null_mut()) != 0 {
        panic!("Failed to write the module to a .ll file");
    } else {
        println!("Module saved to {}", filename);
    }
}}

/// Generate the assembly file from the module.
unsafe fn generate_assembly(module: LLVMModuleRef, filename: &str) { unsafe {
    let c_filename = c_str(filename);
    let target_triple = LLVMGetDefaultTargetTriple();
    let mut target = std::ptr::null_mut();
    let mut error = std::ptr::null_mut();

    if LLVMGetTargetFromTriple(target_triple, &mut target, &mut error) != 0 {
        panic!(
            "Failed to get target: {}",
            std::ffi::CStr::from_ptr(error).to_string_lossy()
        );
    }

    let target_machine = LLVMCreateTargetMachine(
        target,
        target_triple,
        c_str("generic").as_ptr(),
        c_str("").as_ptr(),
        LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
        LLVMRelocMode::LLVMRelocDefault,
        LLVMCodeModel::LLVMCodeModelDefault,
    );

    if LLVMTargetMachineEmitToFile(
        target_machine,
        module,
        c_filename.as_ptr(),
        LLVMAssemblyFile,
        ptr::null_mut(),
    ) != 0
    {
        panic!("Failed to generate assembly");
    } else {
        println!("Assembly saved to {}", filename);
    }

    LLVMDisposeTargetMachine(target_machine);
}}

fn c_str(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn c_str_from_ptr(ptr: *mut i8) -> String {
    unsafe { CString::from_raw(ptr).to_string_lossy().into_owned() }
}
