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
        assert_eq!(
            LLVM_InitializeNativeTarget(),
            0,
            "[LLVM] InitializeNativeTarget failed"
        );
        assert_eq!(
            LLVM_InitializeNativeAsmPrinter(),
            0,
            "[LLVM] InitializeNativeTargetAsmPrinter failed"
        );
        assert_eq!(
            LLVM_InitializeNativeAsmParser(),
            0,
            "[LLVM] InitializeNativeTargetAsmParser failed"
        );

        // Create a new LLVM context and module
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(c_str("hello_module").as_ptr(), context);

        // Create the function signature for main
        let main_func_type =
            LLVMFunctionType(LLVMVoidTypeInContext(context), ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c_str("main").as_ptr(), main_func_type);

        // Create a basic block and builder
        let entry = LLVMAppendBasicBlockInContext(context, main_func, c_str("entry").as_ptr());
        let builder = LLVMCreateBuilderInContext(context);
        LLVMPositionBuilderAtEnd(builder, entry);

        // Create the format string for printf
        let hello_str = c_str("Please enter x and y: ");
        let hello_global =
            LLVMBuildGlobalStringPtr(builder, hello_str.as_ptr(), c_str("hello_str").as_ptr());

        // Create printf function signature
        let printf_func_type = LLVMFunctionType(
            LLVMInt32TypeInContext(context),
            [LLVMPointerType(LLVMInt8TypeInContext(context), 0)].as_mut_ptr(),
            1,
            1,
        );
        let printf_func = LLVMAddFunction(module, c_str("printf").as_ptr(), printf_func_type);

        // Call printf with the format string
        LLVMBuildCall2(
            builder,
            printf_func_type,
            printf_func,
            [hello_global].as_mut_ptr(),
            1,
            c_str("").as_ptr(),
        );

        let scanf_func_type = LLVMFunctionType(
            LLVMInt32TypeInContext(context),
            [
                LLVMPointerType(LLVMInt8TypeInContext(context), 0),
                LLVMPointerType(LLVMInt32TypeInContext(context), 0),
                LLVMPointerType(LLVMInt32TypeInContext(context), 0), // Two integers for scanf
            ]
            .as_mut_ptr(),
            3, // Adjusted to 3 for three parameters
            1,
        );
        let scanf_func = LLVMAddFunction(module, c_str("scanf").as_ptr(), scanf_func_type);

        // Create format string for scanf to read two integers
        let scanf_str = c_str("%d %d");
        let scanf_global =
            LLVMBuildGlobalStringPtr(builder, scanf_str.as_ptr(), c_str("scanf_str").as_ptr());

        // Allocate memory for x and y
        let x = LLVMBuildAlloca(
            builder,
            LLVMInt32TypeInContext(context),
            c_str("x").as_ptr(),
        );
        let y = LLVMBuildAlloca(
            builder,
            LLVMInt32TypeInContext(context),
            c_str("y").as_ptr(),
        );

        // Call scanf to read x and y from the user
        LLVMBuildCall2(
            builder,
            scanf_func_type,
            scanf_func,
            [scanf_global, x, y].as_mut_ptr(),
            3,
            c_str("").as_ptr(),
        );
        // LLVMBuildCall2(
        //     builder,
        //     scanf_func_type,
        //     scanf_func,
        //     [scanf_global, y].as_mut_ptr(),
        //     2,
        //     c_str("").as_ptr(),
        // );

        // Load values from x and y using LLVMBuildLoad2
        let x_loaded = LLVMBuildLoad2(
            builder,
            LLVMInt32TypeInContext(context),
            x,
            c_str("x_val").as_ptr(),
        );
        let y_loaded = LLVMBuildLoad2(
            builder,
            LLVMInt32TypeInContext(context),
            y,
            c_str("y_val").as_ptr(),
        );

        // Add x and y
        let sum = LLVMBuildAdd(builder, x_loaded, y_loaded, c_str("sum").as_ptr());

        // Create the format string for the sum
        let sum_str = c_str("x + y = %d\n");
        let sum_global =
            LLVMBuildGlobalStringPtr(builder, sum_str.as_ptr(), c_str("sum_str").as_ptr());

        // Call printf with the sum result
        LLVMBuildCall2(
            builder,
            printf_func_type,
            printf_func,
            [sum_global, sum].as_mut_ptr(),
            2,
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

        // Generate the target object file
        generate_target(module, "output.o");

        // Link the object file to generate the executable
        link_object_to_executable();

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

// Save the LLVM module to a `.ll` file.
unsafe fn save_module_to_ll(module: LLVMModuleRef, filename: &str) {
    unsafe {
        let c_filename = c_str(filename);
        if LLVMPrintModuleToFile(module, c_filename.as_ptr(), ptr::null_mut()) != 0 {
            panic!("Failed to write the module to a .ll file");
        } else {
            println!("Module saved to {}", filename);
        }
    }
}

// Generate the assembly file from the module.
unsafe fn generate_assembly(module: LLVMModuleRef, filename: &str) {
    unsafe {
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
    }
}

fn c_str(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn c_str_from_ptr(ptr: *mut i8) -> String {
    unsafe { CString::from_raw(ptr).to_string_lossy().into_owned() }
}

// Modify the generate_assembly function to generate a target object file
unsafe fn generate_target(module: LLVMModuleRef, filename: &str) {
    unsafe {
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
            b"generic\0".as_ptr() as *const _,
            b"\0".as_ptr() as *const _,
            LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let output_file = std::ffi::CString::new(filename).unwrap();

        if LLVMTargetMachineEmitToFile(
            target_machine,
            module,
            output_file.as_ptr() as *mut _,
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut error,
        ) != 0
        {
            panic!(
                "Failed to emit object file: {}",
                std::ffi::CStr::from_ptr(error).to_string_lossy()
            );
        }

        println!("Generated object file: {}", filename);

        LLVMDisposeTargetMachine(target_machine);
    }
}

// Link the object file to generate an executable ELF file
fn link_object_to_executable() {
    let output_filename = "output.out";
    let object_filename = "output.o";

    let status = std::process::Command::new("gcc")
        .arg(object_filename)
        .arg("-o")
        .arg(output_filename)
        .status()
        .expect("Failed to execute gcc");

    if status.success() {
        println!("Executable file created: {}", output_filename);
    } else {
        panic!("Linking failed");
    }
}
