use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::LLVMCodeGenFileType::*;
use llvm_sys::target_machine::*;
use std::ffi::CString;
use std::fmt::Display;
use std::ptr;

macro_rules! s_cstr {
    ($s:expr) => {{
        const BUFFER_SIZE: usize = 256;
        let mut buffer = [0u8; BUFFER_SIZE];

        if $s.len() >= BUFFER_SIZE {
            panic!("String is too long, maximum length is {}", BUFFER_SIZE - 1);
        }

        buffer[..$s.len()].copy_from_slice($s.as_bytes());
        buffer[$s.len()] = 0;

        buffer.as_ptr() as *const i8
    }};
}

struct LLVMVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl LLVMVersion {
    fn get_llvm_version() -> Self {
        let mut major: u32 = 0;
        let mut minor: u32 = 0;
        let mut patch: u32 = 0;

        unsafe { LLVMGetVersion(&mut major, &mut minor, &mut patch) };

        Self {
            major,
            minor,
            patch,
        }
    }
}

impl Display for LLVMVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn main() {
    println!("LLVM version: {}", LLVMVersion::get_llvm_version());

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
        let module = LLVMModuleCreateWithNameInContext(c"shizuku_module".as_ptr(), context);

        // Create the function signature for main
        let main_func_type =
            LLVMFunctionType(LLVMVoidTypeInContext(context), ptr::null_mut(), 0, 0);
        let main_func = LLVMAddFunction(module, c"main".as_ptr(), main_func_type);

        // Create a basic block and builder
        let entry = LLVMAppendBasicBlockInContext(context, main_func, c"entry".as_ptr());
        let builder = LLVMCreateBuilderInContext(context);
        LLVMPositionBuilderAtEnd(builder, entry);

        // Create the format string for printf
        let prompt_str = c"Please enter x and y: ";
        let prompt_global =
            LLVMBuildGlobalStringPtr(builder, prompt_str.as_ptr(), c"prompt_str".as_ptr());

        // Create format string for scanf to read two integers
        let scanf_str = c"%d %d";
        let scanf_global =
            LLVMBuildGlobalStringPtr(builder, scanf_str.as_ptr(), c"scanf_str".as_ptr());

        // Create printf function signature
        let printf_func_type = LLVMFunctionType(
            LLVMInt32TypeInContext(context),
            [LLVMPointerType(LLVMInt8TypeInContext(context), 0)].as_mut_ptr(),
            1,
            1,
        );
        let printf_func = LLVMAddFunction(module, c"printf".as_ptr(), printf_func_type);

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
        let scanf_func = LLVMAddFunction(module, c"scanf".as_ptr(), scanf_func_type);

        // Allocate memory for x and y
        let x = LLVMBuildAlloca(builder, LLVMInt32TypeInContext(context), c"x".as_ptr());
        let y = LLVMBuildAlloca(builder, LLVMInt32TypeInContext(context), c"y".as_ptr());

        let loop_cond_block =
            LLVMAppendBasicBlockInContext(context, main_func, c"loop_cond".as_ptr());
        let loop_body_block =
            LLVMAppendBasicBlockInContext(context, main_func, c"loop_body".as_ptr());
        let loop_exit_block =
            LLVMAppendBasicBlockInContext(context, main_func, c"loop_exit".as_ptr());

        // Jump to loop condition block from entry
        LLVMBuildBr(builder, loop_cond_block);

        // Set up the loop condition block
        LLVMPositionBuilderAtEnd(builder, loop_cond_block);

        // Call printf with the format string
        LLVMBuildCall2(
            builder,
            printf_func_type,
            printf_func,
            [prompt_global].as_mut_ptr(),
            1,
            c"".as_ptr(),
        );

        // Call scanf to read x and y from the user
        LLVMBuildCall2(
            builder,
            scanf_func_type,
            scanf_func,
            [scanf_global, x, y].as_mut_ptr(),
            3,
            c"".as_ptr(),
        );

        let x_loaded = LLVMBuildLoad2(
            builder,
            LLVMInt32TypeInContext(context),
            x,
            c"x_val".as_ptr(),
        );
        let y_loaded = LLVMBuildLoad2(
            builder,
            LLVMInt32TypeInContext(context),
            y,
            c"y_val".as_ptr(),
        );

        // Add x and y
        let sum = LLVMBuildAdd(builder, x_loaded, y_loaded, c"sum".as_ptr());

        let condition = LLVMBuildICmp(
            builder,
            llvm_sys::LLVMIntPredicate::LLVMIntEQ,
            sum,
            LLVMConstInt(LLVMInt32TypeInContext(context), 15, 0),
            c"is_equal".as_ptr(),
        );
        LLVMBuildCondBr(builder, condition, loop_exit_block, loop_body_block);

        // Set up the loop body block
        LLVMPositionBuilderAtEnd(builder, loop_body_block);

        // Create the format string for the sum
        let sum_str = c"sum is %d\n";
        let sum_global = LLVMBuildGlobalStringPtr(builder, sum_str.as_ptr(), c"sum_str".as_ptr());

        // Call printf with the sum result
        LLVMBuildCall2(
            builder,
            printf_func_type,
            printf_func,
            [sum_global, sum].as_mut_ptr(),
            2,
            c"".as_ptr(),
        );

        // Jump back to the condition check
        LLVMBuildBr(builder, loop_cond_block);

        // Set up the loop exit block
        LLVMPositionBuilderAtEnd(builder, loop_exit_block);

        // Print success message
        let success_str = c"Success: x + y = 15\n";
        let success_global =
            LLVMBuildGlobalStringPtr(builder, success_str.as_ptr(), c"success_str".as_ptr());
        LLVMBuildCall2(
            builder,
            printf_func_type,
            printf_func,
            [success_global].as_mut_ptr(),
            1,
            c"".as_ptr(),
        );

        // Return void
        LLVMBuildRetVoid(builder);

        // Verify the module
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMAbortProcessAction,
            ptr::null_mut(),
        );

        // Save the module to a .ll file
        save_module_to_ll(module, "a.ll");

        // Generate assembly from the module
        generate_assembly(module, "a.s");

        // Generate the target object file
        generate_target(module, "a.o");

        // Link the object file to generate the executable
        link_object_to_executable("a.o", "a.out");

        // JIT compile and execute
        let mut engine: LLVMExecutionEngineRef = ptr::null_mut();
        let mut error: *mut i8 = ptr::null_mut();
        if LLVMCreateJITCompilerForModule(&mut engine, module, 0, &mut error) != 0 {
            panic!("Failed to create JIT compiler: {}", c_str_from_ptr(error));
        }

        let main_func = LLVMGetNamedFunction(module, c"main".as_ptr());
        LLVMRunFunction(engine, main_func, 0, ptr::null_mut());

        // Clean up
        LLVMDisposeBuilder(builder);
        LLVMDisposeExecutionEngine(engine);
        LLVMContextDispose(context);
    }
}

// Save the LLVM module to a `.ll` file.
fn save_module_to_ll(module: LLVMModuleRef, filename: &str) {
    unsafe {
        if LLVMPrintModuleToFile(module, s_cstr!(filename), ptr::null_mut()) != 0 {
            panic!("Failed to write the module to a .ll file");
        } else {
            println!("Module saved to {}", filename);
        }
    }
}

// Generate the assembly file from the module.
fn generate_assembly(module: LLVMModuleRef, filename: &str) {
    unsafe {
        let c_filename = s_cstr!(filename);
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
            c"generic".as_ptr(),
            c"".as_ptr(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        if LLVMTargetMachineEmitToFile(
            target_machine,
            module,
            c_filename,
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

// #[inline]
// fn c_str(s: &str) -> *const i8{
//     // return CString::new(s).unwrap();
//     let mut buffer = [0u8; 256];

//     if s.len() >= buffer.len() {
//         panic!(
//             "Filename is too long, maximum length is {}",
//             buffer.len() - 1
//         );
//     }

//     buffer[..s.len()].copy_from_slice(s.as_bytes());
//     buffer[s.len()] = 0; // Null terminator

//     let c_s = buffer.as_ptr() as *const i8;
//     return c_s
// }

fn c_str_from_ptr(ptr: *mut i8) -> String {
    unsafe { CString::from_raw(ptr).to_string_lossy().into_owned() }
}

// Modify the generate_assembly function to generate a target object file
fn generate_target(module: LLVMModuleRef, filename: &str) {
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
            c"generic".as_ptr(),
            c"".as_ptr(),
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
fn link_object_to_executable(object_filename: &str, output_filename: &str) {

    let status = std::process::Command::new("gcc")
        .arg(object_filename)
        .arg("-o")
        .arg(output_filename)
        .arg("-no-pie")
        .status()
        .expect("Failed to execute gcc");

    if status.success() {
        println!("Executable file created: {}", output_filename);
    } else {
        panic!("Linking failed");
    }
}
