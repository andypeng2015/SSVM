//! Defines WasmEdge Instance and other relevant types.

use crate::{
    error::{InstanceError, WasmEdgeError},
    ffi,
    instance::{function::InnerFunc, global::InnerGlobal, memory::InnerMemory, table::InnerTable},
    types::WasmEdgeString,
    utils::string_to_c_char,
    Function, Global, Memory, Table, WasmEdgeResult,
};

/// An [Instance] represents an instantiated module. In the instantiation process, An [Instance] is created from al[Module](crate::Module). From an [Instance] the exported [functions](crate::Function), [tables](crate::Table), [memories](crate::Memory), and [globals](crate::Global) can be fetched.
///
/// A module instance is usually returned via one of the following APIs:
///
/// * [Executor](crate::Executor)
///     * [Executor::register_named_module](crate::Executor::register_named_module) ([example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/executor_register_named_module.rs))
///     * [Executor::register_active_module](crate::Executor::register_active_module) ([example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/executor_register_active_module.rs))
/// * [Vm](crate::Vm)
///     * [Vm::active_module](crate::Vm::active_module) ([example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/vm_get_active_module.rs))
/// * [Store](crate::Store)
///     * [Store::module](crate::Store::module) ([example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/vm_get_active_module.rs))
///
#[derive(Debug)]
pub struct Instance {
    pub(crate) inner: InnerInstance,
    pub(crate) registered: bool,
}
impl Drop for Instance {
    fn drop(&mut self) {
        if !self.registered && !self.inner.0.is_null() {
            unsafe {
                ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
            }
        }
    }
}
impl Instance {
    /// Returns the name of this exported [module instance](crate::Instance).
    ///
    /// If this module instance is an active module instance, then None is returned.
    pub fn name(&self) -> Option<String> {
        let name = unsafe { ffi::WasmEdge_ModuleInstanceGetModuleName(self.inner.0 as *const _) };

        let name: String = name.into();
        if name.is_empty() {
            return None;
        }

        Some(name)
    }

    /// Returns the exported [function instance](crate::Function) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [function instance](crate::Function).
    ///
    /// # Error
    ///
    /// If fail to find the target [function](crate::Function), then an error is returned.
    pub fn get_func(&self, name: impl AsRef<str>) -> WasmEdgeResult<Function> {
        let func_name: WasmEdgeString = name.as_ref().into();
        let func_ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceFindFunction(self.inner.0 as *const _, func_name.as_raw())
        };
        match func_ctx.is_null() {
            true => Err(WasmEdgeError::Instance(InstanceError::NotFoundFunc(
                name.as_ref().to_string(),
            ))),
            false => Ok(Function {
                inner: InnerFunc(func_ctx),
                registered: true,
            }),
        }
    }

    /// Returns the exported [table instance](crate::Table) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [table instance](crate::Table).
    ///
    /// # Error
    ///
    /// If fail to find the target [table instance](crate::Table), then an error is returned.
    pub fn get_table(&self, name: impl AsRef<str>) -> WasmEdgeResult<Table> {
        let table_name: WasmEdgeString = name.as_ref().into();
        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceFindTable(self.inner.0 as *const _, table_name.as_raw())
        };
        match ctx.is_null() {
            true => Err(WasmEdgeError::Instance(InstanceError::NotFoundTable(
                name.as_ref().to_string(),
            ))),
            false => Ok(Table {
                inner: InnerTable(ctx),
                registered: true,
            }),
        }
    }

    /// Returns the exported [memory instance](crate::Memory) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [memory instance](crate::Memory).
    ///
    /// # Error
    ///
    /// If fail to find the target [memory instance](crate::Memory), then an error is returned.
    pub fn get_memory(&self, name: impl AsRef<str>) -> WasmEdgeResult<Memory> {
        let mem_name: WasmEdgeString = name.as_ref().into();
        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceFindMemory(self.inner.0 as *const _, mem_name.as_raw())
        };
        match ctx.is_null() {
            true => Err(WasmEdgeError::Instance(InstanceError::NotFoundMem(
                name.as_ref().to_string(),
            ))),
            false => Ok(Memory {
                inner: InnerMemory(ctx),
                registered: true,
            }),
        }
    }

    /// Returns the exported [global instance](crate::Global) by name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported [global instance](crate::Global).
    ///
    /// # Error
    ///
    /// If fail to find the target [global instance](crate::Global), then an error is returned.
    pub fn get_global(&self, name: impl AsRef<str>) -> WasmEdgeResult<Global> {
        let global_name: WasmEdgeString = name.as_ref().into();
        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceFindGlobal(self.inner.0 as *const _, global_name.as_raw())
        };
        match ctx.is_null() {
            true => Err(WasmEdgeError::Instance(InstanceError::NotFoundGlobal(
                name.as_ref().to_string(),
            ))),
            false => Ok(Global {
                inner: InnerGlobal(ctx),
                registered: true,
            }),
        }
    }

    /// Returns the length of the exported [function instances](crate::Function) in this module instance.
    pub fn func_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListFunctionLength(self.inner.0) }
    }

    /// Returns the names of the exported [function instances](crate::Function) in this module instance.
    pub fn func_names(&self) -> Option<Vec<String>> {
        let len_func_names = self.func_len();
        match len_func_names > 0 {
            true => {
                let mut func_names = Vec::with_capacity(len_func_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListFunction(
                        self.inner.0,
                        func_names.as_mut_ptr(),
                        len_func_names,
                    );
                    func_names.set_len(len_func_names as usize);
                }

                let names = func_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the length of the exported [table instances](crate::Table) in this module instance.
    pub fn table_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListTableLength(self.inner.0) }
    }

    /// Returns the names of the exported [table instances](crate::Table) in this module instance.
    pub fn table_names(&self) -> Option<Vec<String>> {
        let len_table_names = self.table_len();
        match len_table_names > 0 {
            true => {
                let mut table_names = Vec::with_capacity(len_table_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListTable(
                        self.inner.0,
                        table_names.as_mut_ptr(),
                        len_table_names,
                    );
                    table_names.set_len(len_table_names as usize);
                }

                let names = table_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the length of the exported [memory instances](crate::Memory) in this module instance.
    pub fn mem_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListMemoryLength(self.inner.0) }
    }

    /// Returns the names of all exported [memory instances](crate::Memory) in this module instance.
    pub fn mem_names(&self) -> Option<Vec<String>> {
        let len_mem_names = self.mem_len();
        match len_mem_names > 0 {
            true => {
                let mut mem_names = Vec::with_capacity(len_mem_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListMemory(
                        self.inner.0,
                        mem_names.as_mut_ptr(),
                        len_mem_names,
                    );
                    mem_names.set_len(len_mem_names as usize);
                }

                let names = mem_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }

    /// Returns the length of the exported [global instances](crate::Global) in this module instance.
    pub fn global_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceListGlobalLength(self.inner.0) }
    }

    /// Returns the names of the exported [global instances](crate::Global) in this module instance.
    pub fn global_names(&self) -> Option<Vec<String>> {
        let len_global_names = self.global_len();
        match len_global_names > 0 {
            true => {
                let mut global_names = Vec::with_capacity(len_global_names as usize);
                unsafe {
                    ffi::WasmEdge_ModuleInstanceListGlobal(
                        self.inner.0,
                        global_names.as_mut_ptr(),
                        len_global_names,
                    );
                    global_names.set_len(len_global_names as usize);
                }

                let names = global_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct InnerInstance(pub(crate) *mut ffi::WasmEdge_ModuleInstanceContext);
unsafe impl Send for InnerInstance {}
unsafe impl Sync for InnerInstance {}

/// An [ImportModule] represents a host module with a name. A host module consists of one or more host [function](crate::Function), [table](crate::Table), [memory](crate::Memory), and [global](crate::Global) instances,  which are defined outside wasm modules and fed into wasm modules as imports.
///
/// # Example
///
/// The following example shows how to use [ImportModule] to import [host function](crate::Function), [table](crate::Table), [memory](crate::Memory) and [global](crate::Global) instances, and to register it into [Vm](crate::Vm).
///
/// ```rust
/// use wasmedge_sys::{
///     ImportInstance, FuncType, Function, Global, GlobalType, ImportModule, ImportObject, MemType,
///     Memory, Table, TableType, Vm, WasmValue,
/// };
/// use wasmedge_types::{Mutability, RefType, ValType};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let module_name = "extern_module";
///
///     // create ImportModule instance
///     let mut import = ImportModule::create(module_name)?;
///
///     // a function to import
///     fn real_add(inputs: Vec<WasmValue>) -> Result<Vec<WasmValue>, u8> {
///         if inputs.len() != 2 {
///             return Err(1);
///         }
///
///         let a = if inputs[0].ty() == ValType::I32 {
///             inputs[0].to_i32()
///         } else {
///             return Err(2);
///         };
///
///         let b = if inputs[1].ty() == ValType::I32 {
///             inputs[1].to_i32()
///         } else {
///             return Err(3);
///         };
///
///         let c = a + b;
///
///         Ok(vec![WasmValue::from_i32(c)])
///     }
///
///     // add host function
///     let func_ty = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32])?;
///     let host_func = Function::create(&func_ty, Box::new(real_add), 0)?;
///     import.add_func("add", host_func);
///
///     // add table
///     let table_ty = TableType::create(RefType::FuncRef, 0..=u32::MAX)?;
///     let table = Table::create(&table_ty)?;
///     import.add_table("table", table);
///
///     // add memory
///     let mem_ty = MemType::create(0..=u32::MAX)?;
///     let memory = Memory::create(&mem_ty)?;
///     import.add_memory("mem", memory);
///
///     // add global
///     let ty = GlobalType::create(ValType::F32, Mutability::Const)?;
///     let global = Global::create(&ty, WasmValue::from_f32(3.5))?;
///     import.add_global("global", global);
///
///     let mut vm = Vm::create(None, None)?;
///
///     vm.register_wasm_from_import(ImportObject::Import(import))?;
///
///     Ok(())
/// }
///
/// ```
///  
#[derive(Debug)]
pub struct ImportModule {
    pub(crate) inner: InnerInstance,
    pub(crate) registered: bool,
    name: String,
}
impl Drop for ImportModule {
    fn drop(&mut self) {
        if !self.registered && !self.inner.0.is_null() {
            unsafe {
                ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
            }
        }
    }
}
impl ImportModule {
    /// Creates a module instance which is used to import host functions, tables, memories, and globals into a wasm module.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the import module instance.
    ///
    /// # Error
    ///
    /// If fail to create the import module instance, then an error is returned.
    pub fn create(name: impl AsRef<str>) -> WasmEdgeResult<Self> {
        let raw_name = WasmEdgeString::from(name.as_ref());
        let ctx = unsafe { ffi::WasmEdge_ModuleInstanceCreate(raw_name.as_raw()) };

        match ctx.is_null() {
            true => Err(WasmEdgeError::Instance(InstanceError::CreateImportModule)),
            false => Ok(Self {
                inner: InnerInstance(ctx),
                registered: false,
                name: name.as_ref().to_string(),
            }),
        }
    }

    /// Returns the name of this import module instance.
    pub fn name(&self) -> String {
        self.name.to_owned()
    }
}
impl ImportInstance for ImportModule {
    fn add_func(&mut self, name: impl AsRef<str>, mut func: Function) {
        let func_name: WasmEdgeString = name.into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddFunction(self.inner.0, func_name.as_raw(), func.inner.0);
        }
        func.inner.0 = std::ptr::null_mut();
    }

    fn add_table(&mut self, name: impl AsRef<str>, mut table: Table) {
        let table_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddTable(self.inner.0, table_name.as_raw(), table.inner.0);
        }
        table.inner.0 = std::ptr::null_mut();
    }

    fn add_memory(&mut self, name: impl AsRef<str>, mut memory: Memory) {
        let mem_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddMemory(self.inner.0, mem_name.as_raw(), memory.inner.0);
        }
        memory.inner.0 = std::ptr::null_mut();
    }

    fn add_global(&mut self, name: impl AsRef<str>, mut global: Global) {
        let global_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddGlobal(
                self.inner.0,
                global_name.as_raw(),
                global.inner.0,
            );
        }
        global.inner.0 = std::ptr::null_mut();
    }
}

/// A [WasiModule] is a module instance for the WASI specification.
///
/// # Usage
///
/// * [WasiModule] implements [ImportInstance](crate::ImportInstance) trait, therefore it can be used to register function, table, memory and global instances.
///     * [Example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/wasi_module.rs)
///
/// * A [WasiModule] can be created implicitly inside a [Vm](crate::Vm) by passing the [Vm](crate::Vm) a [config](crate::Config) argument in which the `wasi` option is enabled.
///    * [Example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/wasi_module.rs)
///
#[derive(Debug)]
pub struct WasiModule {
    pub(crate) inner: InnerInstance,
    pub(crate) registered: bool,
}
impl Drop for WasiModule {
    fn drop(&mut self) {
        if !self.registered && !self.inner.0.is_null() {
            unsafe {
                ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
            }
        }
    }
}
impl WasiModule {
    /// Creates a WASI host module which contains the WASI host functions, and initializes it with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables in the format `ENV_VAR_NAME=VALUE`.
    ///
    /// * `preopens` - The directories to pre-open. The required format is `DIR1:DIR2`.
    ///
    /// # Error
    ///
    /// If fail to create a host module, then an error is returned.
    pub fn create(
        args: Option<Vec<&str>>,
        envs: Option<Vec<&str>>,
        preopens: Option<Vec<&str>>,
    ) -> WasmEdgeResult<Self> {
        let args = match args {
            Some(args) => args.into_iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let args_len = args.len();

        let envs = match envs {
            Some(envs) => envs.into_iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let envs_len = envs.len();

        let preopens = match preopens {
            Some(preopens) => preopens
                .into_iter()
                .map(string_to_c_char)
                .collect::<Vec<_>>(),
            None => vec![],
        };
        let preopens_len = preopens.len();

        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceCreateWASI(
                args.as_ptr(),
                args_len as u32,
                envs.as_ptr(),
                envs_len as u32,
                preopens.as_ptr(),
                preopens_len as u32,
            )
        };
        match ctx.is_null() {
            true => Err(WasmEdgeError::ImportObjCreate),
            false => Ok(Self {
                inner: InnerInstance(ctx),
                registered: false,
            }),
        }
    }

    /// Returns the name of this wasi module instance.
    pub fn name(&self) -> String {
        String::from("wasi_snapshot_preview1")
    }

    /// Initializes the WASI host module with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables in the format `ENV_VAR_NAME=VALUE`.
    ///
    /// * `preopens` - The directories to pre-open. The required format is `DIR1:DIR2`.
    pub fn init_wasi(
        &mut self,
        args: Option<Vec<&str>>,
        envs: Option<Vec<&str>>,
        preopens: Option<Vec<&str>>,
    ) {
        let args = match args {
            Some(args) => args.into_iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let args_len = args.len();

        let envs = match envs {
            Some(envs) => envs.into_iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let envs_len = envs.len();

        let preopens = match preopens {
            Some(preopens) => preopens
                .into_iter()
                .map(string_to_c_char)
                .collect::<Vec<_>>(),
            None => vec![],
        };
        let preopens_len = preopens.len();

        unsafe {
            ffi::WasmEdge_ModuleInstanceInitWASI(
                self.inner.0,
                args.as_ptr(),
                args_len as u32,
                envs.as_ptr(),
                envs_len as u32,
                preopens.as_ptr(),
                preopens_len as u32,
            )
        };
    }

    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        unsafe { ffi::WasmEdge_ModuleInstanceWASIGetExitCode(self.inner.0 as *const _) }
    }
}
impl ImportInstance for WasiModule {
    fn add_func(&mut self, name: impl AsRef<str>, mut func: Function) {
        let func_name: WasmEdgeString = name.into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddFunction(self.inner.0, func_name.as_raw(), func.inner.0);
        }
        func.inner.0 = std::ptr::null_mut();
    }

    fn add_table(&mut self, name: impl AsRef<str>, mut table: Table) {
        let table_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddTable(self.inner.0, table_name.as_raw(), table.inner.0);
        }
        table.inner.0 = std::ptr::null_mut();
    }

    fn add_memory(&mut self, name: impl AsRef<str>, mut memory: Memory) {
        let mem_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddMemory(self.inner.0, mem_name.as_raw(), memory.inner.0);
        }
        memory.inner.0 = std::ptr::null_mut();
    }

    fn add_global(&mut self, name: impl AsRef<str>, mut global: Global) {
        let global_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddGlobal(
                self.inner.0,
                global_name.as_raw(),
                global.inner.0,
            );
        }
        global.inner.0 = std::ptr::null_mut();
    }
}

/// A [WasmEdgeProcessModule] is a module instance for the WasmEdge_Process specification.
///
/// Notice that before creating or initiating a [WasmEdgeProcessModule], it MUST be guaranteed that the `wasmedge_process` plugins are loaded. If not, use the [load_plugin_from_default_paths](crate::utils::load_plugin_from_default_paths) function to load the relevant plugins from the default paths, shown as the code below.
///
/// ```rust
/// use wasmedge_sys::{utils, WasmEdgeProcessModule};
///
/// // load plugins from default paths
/// utils::load_plugin_from_default_paths();
///
/// // create wasmedge_process
/// let result = WasmEdgeProcessModule::create(Some(vec!["arg1", "arg2"]), true);
/// assert!(result.is_ok());
/// ```
///
///
/// # Usage
///
/// * [WasmEdgeProcessModule] implements [ImportInstance](crate::ImportInstance) trait, therefore it can be used to register function, table, memory and global instances.
///     * [Example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/wasmedge_process_module.rs)
///
/// * A [WasmEdgeProcessModule] can be created implicitly inside a [Vm](crate::Vm) by passing the [Vm](crate::Vm) a [config](crate::Config) argument in which the `wasmedge_process` option is enabled.
///     * [Example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/wasmedge_process_module.rs)
///
#[derive(Debug)]
pub struct WasmEdgeProcessModule {
    pub(crate) inner: InnerInstance,
    pub(crate) registered: bool,
}
impl Drop for WasmEdgeProcessModule {
    fn drop(&mut self) {
        if !self.registered && !self.inner.0.is_null() {
            unsafe {
                ffi::WasmEdge_ModuleInstanceDelete(self.inner.0);
            }
        }
    }
}
impl WasmEdgeProcessModule {
    /// Creates a wasmedge_process host module that contains the wasmedge_process host functions and
    /// initialize it with the parameters.
    ///
    /// # Arguments
    ///
    /// * `allowed_cmds` - A white list of commands.
    ///
    /// * `allowed` - Determines if wasmedge_process is allowed to execute all commands on the white list.
    ///
    /// # Error
    ///
    /// If fail to create a wasmedge_process host module, then an error is returned.
    pub fn create(allowed_cmds: Option<Vec<&str>>, allowed: bool) -> WasmEdgeResult<Self> {
        let cmds = match allowed_cmds {
            Some(cmds) => cmds.iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let cmds_len = cmds.len();

        let ctx = unsafe {
            ffi::WasmEdge_ModuleInstanceCreateWasmEdgeProcess(
                cmds.as_ptr(),
                cmds_len as u32,
                allowed,
            )
        };
        match ctx.is_null() {
            true => Err(WasmEdgeError::ImportObjCreate),
            false => Ok(Self {
                inner: InnerInstance(ctx),
                registered: false,
            }),
        }
    }

    /// Returns the name of this wasmedge_process module instance.
    pub fn name(&self) -> String {
        String::from("wasmedge_process")
    }

    /// Initializes the wasmedge_process host module with the parameters.
    ///
    /// # Arguments
    ///
    /// * `allowed_cmds` - A white list of commands.
    ///
    /// * `allowed` - Determines if wasmedge_process is allowed to execute all commands on the white list.
    pub fn init_wasmedge_process(&mut self, allowed_cmds: Option<Vec<&str>>, allowed: bool) {
        let cmds = match allowed_cmds {
            Some(cmds) => cmds.iter().map(string_to_c_char).collect::<Vec<_>>(),
            None => vec![],
        };
        let cmds_len = cmds.len();

        unsafe {
            ffi::WasmEdge_ModuleInstanceInitWasmEdgeProcess(cmds.as_ptr(), cmds_len as u32, allowed)
        }
    }
}
impl ImportInstance for WasmEdgeProcessModule {
    fn add_func(&mut self, name: impl AsRef<str>, mut func: Function) {
        let func_name: WasmEdgeString = name.into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddFunction(self.inner.0, func_name.as_raw(), func.inner.0);
        }
        func.inner.0 = std::ptr::null_mut();
    }

    fn add_table(&mut self, name: impl AsRef<str>, mut table: Table) {
        let table_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddTable(self.inner.0, table_name.as_raw(), table.inner.0);
        }
        table.inner.0 = std::ptr::null_mut();
    }

    fn add_memory(&mut self, name: impl AsRef<str>, mut memory: Memory) {
        let mem_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddMemory(self.inner.0, mem_name.as_raw(), memory.inner.0);
        }
        memory.inner.0 = std::ptr::null_mut();
    }

    fn add_global(&mut self, name: impl AsRef<str>, mut global: Global) {
        let global_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            ffi::WasmEdge_ModuleInstanceAddGlobal(
                self.inner.0,
                global_name.as_raw(),
                global.inner.0,
            );
        }
        global.inner.0 = std::ptr::null_mut();
    }
}

/// The object to be registered into a [Vm](crate::Vm) or an [Executor](crate::Executor) instance is required to implement this trait. The object that implements this trait can be registered via the [Vm::register_wasm_from_import](crate::Vm::register_wasm_from_import) function, or the [Executor::register_import_object](crate::Executor::register_import_object) function.
pub trait ImportInstance {
    /// Imports a [host function instance](crate::Function).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the host function instance to import.
    ///
    /// * `func` - The host function instance to import.
    fn add_func(&mut self, name: impl AsRef<str>, func: Function);

    /// Imports a [table instance](crate::Table).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the host table instance to import.
    ///
    /// * `table` - The host table instance to import.
    fn add_table(&mut self, name: impl AsRef<str>, table: Table);

    /// Imports a [memory instance](crate::Memory).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the host memory instance to import.
    ///
    /// * `memory` - The host memory instance to import.
    fn add_memory(&mut self, name: impl AsRef<str>, memory: Memory);

    /// Imports a [global instance](crate::Global).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the host global instance to import.
    ///
    /// * `global` - The host global instance to import.
    fn add_global(&mut self, name: impl AsRef<str>, global: Global);
}

/// Defines three types of module instances that can be imported into a WasmEdge [Store](crate::Store) instance.
#[derive(Debug)]
pub enum ImportObject {
    /// Defines the import module instance is of ImportModule type.
    Import(ImportModule),
    /// Defines the import module instance is of WasiModule type.
    Wasi(WasiModule),
    /// Defines the import module instance is of WasmEdgeProcessModule type.
    WasmEdgeProcess(WasmEdgeProcessModule),
}
impl ImportObject {
    /// Returns the name of the import object.
    pub fn name(&self) -> String {
        match self {
            ImportObject::Import(import) => import.name(),
            ImportObject::Wasi(wasi) => wasi.name(),
            ImportObject::WasmEdgeProcess(wasmedge_process) => wasmedge_process.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        utils, Config, Executor, FuncType, GlobalType, ImportModule, MemType, Store, TableType, Vm,
        WasmValue,
    };
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    use wasmedge_types::{Mutability, RefType, ValType};

    #[test]
    fn test_instance_add_instance() {
        let host_name = "extern";

        // create an import module
        let result = ImportModule::create(host_name);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // create a host function
        let result = FuncType::create([ValType::ExternRef, ValType::I32], [ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create(&func_ty, Box::new(real_add), 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        // add the host function
        import.add_func("func-add", host_func);

        // create a table
        let result = TableType::create(RefType::FuncRef, 10..=20);
        assert!(result.is_ok());
        let table_ty = result.unwrap();
        let result = Table::create(&table_ty);
        assert!(result.is_ok());
        let host_table = result.unwrap();
        // add the table
        import.add_table("table", host_table);

        // create a memory
        let result = MemType::create(1..=2);
        assert!(result.is_ok());
        let mem_ty = result.unwrap();
        let result = Memory::create(&mem_ty);
        assert!(result.is_ok());
        let host_memory = result.unwrap();
        // add the memory
        import.add_memory("memory", host_memory);

        // create a global
        let result = GlobalType::create(ValType::I32, Mutability::Const);
        assert!(result.is_ok());
        let global_ty = result.unwrap();
        let result = Global::create(&global_ty, WasmValue::from_i32(666));
        assert!(result.is_ok());
        let host_global = result.unwrap();
        // add the global
        import.add_global("global_i32", host_global);
    }

    #[test]
    fn test_instance_import_module_send() {
        let host_name = "extern";

        // create an ImportModule instance
        let result = ImportModule::create(host_name);
        assert!(result.is_ok());
        let import = result.unwrap();

        let handle = thread::spawn(move || {
            assert!(!import.inner.0.is_null());
            println!("{:?}", import.inner);
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_instance_import_module_sync() {
        let host_name = "extern";

        // create an ImportModule instance
        let result = ImportModule::create(host_name);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create(&func_ty, Box::new(real_add), 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0..=u32::MAX);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let memory = {
            let result = MemType::create(10..=20);
            assert!(result.is_ok());
            let mem_ty = result.unwrap();
            let result = Memory::create(&mem_ty);
            assert!(result.is_ok());
            result.unwrap()
        };
        import.add_memory("memory", memory);

        // add globals
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        let import = ImportObject::Import(import);
        let import = Arc::new(Mutex::new(import));
        let import_cloned = Arc::clone(&import);
        let handle = thread::spawn(move || {
            let result = import_cloned.lock();
            assert!(result.is_ok());
            let import = result.unwrap();

            // create a store
            let result = Store::create();
            assert!(result.is_ok());
            let mut store = result.unwrap();
            assert!(!store.inner.0.is_null());
            assert!(!store.registered);

            // create an executor
            let result = Config::create();
            assert!(result.is_ok());
            let config = result.unwrap();
            let result = Executor::create(Some(config), None);
            assert!(result.is_ok());
            let mut executor = result.unwrap();

            // register import object into store
            let result = executor.register_import_object(&mut store, &import);
            assert!(result.is_ok());

            // get the exported module by name
            let result = store.module("extern");
            assert!(result.is_ok());
            let instance = result.unwrap();

            // get the exported function by name
            let result = instance.get_func("add");
            assert!(result.is_ok());

            // get the exported global by name
            let result = instance.get_global("global");
            assert!(result.is_ok());
            let global = result.unwrap();
            assert!(!global.inner.0.is_null() && global.registered);
            let val = global.get_value();
            assert_eq!(val.to_f32(), 3.5);

            // get the exported memory by name
            let result = instance.get_memory("memory");
            assert!(result.is_ok());
            let memory = result.unwrap();
            let result = memory.ty();
            assert!(result.is_ok());
            let ty = result.unwrap();
            assert_eq!(ty.limit(), 10..=20);

            // get the exported table by name
            let result = instance.get_table("table");
            assert!(result.is_ok());
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_instance_wasi() {
        // create a wasi module instance
        {
            let result = WasiModule::create(None, None, None);
            assert!(result.is_ok());

            let result = WasiModule::create(
                Some(vec!["arg1", "arg2"]),
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());

            let result = WasiModule::create(
                None,
                Some(vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"]),
                Some(vec![
                    "apiTestData",
                    "Makefile",
                    "CMakeFiles",
                    "ssvmAPICoreTests",
                    ".:.",
                ]),
            );
            assert!(result.is_ok());
            let wasi_import = result.unwrap();
            assert_eq!(wasi_import.exit_code(), 0);
        }

        // initialize WASI in VM
        {
            let result = Config::create();
            assert!(result.is_ok());
            let mut config = result.unwrap();
            config.wasi(true);
            let result = Vm::create(Some(config), None);
            assert!(result.is_ok());
            let mut vm = result.unwrap();

            // get the ImportObject module from vm
            let result = vm.wasi_module_mut();
            assert!(result.is_ok());
            let mut import_wasi = result.unwrap();

            let args = vec!["arg1", "arg2"];
            let envs = vec!["ENV1=VAL1", "ENV1=VAL2", "ENV3=VAL3"];
            let preopens = vec![
                "apiTestData",
                "Makefile",
                "CMakeFiles",
                "ssvmAPICoreTests",
                ".:.",
            ];
            import_wasi.init_wasi(Some(args), Some(envs), Some(preopens));

            assert_eq!(import_wasi.exit_code(), 0);
        }
    }

    #[test]
    fn test_instance_wasmedge_process() {
        // load plugins
        utils::load_plugin_from_default_paths();

        // create wasmedge_process
        {
            let result = WasmEdgeProcessModule::create(Some(vec!["arg1", "arg2"]), true);
            assert!(result.is_ok());

            let result = WasmEdgeProcessModule::create(None, false);
            assert!(result.is_ok());

            let result = WasmEdgeProcessModule::create(Some(vec!["arg1", "arg2"]), false);
            assert!(result.is_ok());
        }

        // initialize wasmedge_process in VM
        {
            let result = Config::create();
            assert!(result.is_ok());
            let mut config = result.unwrap();
            config.wasmedge_process(true);
            assert!(config.wasmedge_process_enabled());
            let result = Vm::create(Some(config), None);
            assert!(result.is_ok());
            let mut vm = result.unwrap();

            let result = vm.wasmedge_process_module_mut();
            assert!(result.is_ok());
            let mut import_wasmedge_process = result.unwrap();
            import_wasmedge_process.init_wasmedge_process(Some(vec!["arg1", "arg2"]), false);
        }
    }

    #[test]
    fn test_instance_find_xxx() {
        let vm = create_vm();
        let result = vm.store_mut();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // get the module named "extern"
        let result = store.module("extern_module");
        assert!(result.is_ok());
        let instance = result.unwrap();

        // check the name of the module
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern_module");

        // get the exported function named "fib"
        let result = instance.get_func("add");
        assert!(result.is_ok());
        let func = result.unwrap();

        // check the type of the function
        let result = func.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();

        // check the parameter types
        let param_types = ty.params_type_iter().collect::<Vec<ValType>>();
        assert_eq!(param_types, [ValType::I32, ValType::I32]);

        // check the return types
        let return_types = ty.returns_type_iter().collect::<Vec<ValType>>();
        assert_eq!(return_types, [ValType::I32]);

        // get the exported table named "table"
        let result = instance.get_table("table");
        assert!(result.is_ok());
        let table = result.unwrap();

        // check the type of the table
        let result = table.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.elem_ty(), RefType::FuncRef);
        assert_eq!(ty.limit(), 0..=u32::MAX);

        // get the exported memory named "mem"
        let result = instance.get_memory("mem");
        assert!(result.is_ok());
        let memory = result.unwrap();

        // check the type of the memory
        let result = memory.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.limit(), 0..=u32::MAX);

        // get the exported global named "global"
        let result = instance.get_global("global");
        assert!(result.is_ok());
        let global = result.unwrap();

        // check the type of the global
        let result = global.ty();
        assert!(result.is_ok());
        let global = result.unwrap();
        assert_eq!(global.value_type(), ValType::F32);
        assert_eq!(global.mutability(), Mutability::Const);
    }

    #[test]
    fn test_instance_find_names() {
        let vm = create_vm();
        let result = vm.store_mut();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // get the module named "extern"
        let result = store.module("extern_module");
        assert!(result.is_ok());
        let instance = result.unwrap();

        // check the name of the module
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern_module");

        assert_eq!(instance.func_len(), 1);
        let result = instance.func_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["add"]);

        assert_eq!(instance.table_len(), 1);
        let result = instance.table_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["table"]);

        assert_eq!(instance.mem_len(), 1);
        let result = instance.mem_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["mem"]);

        assert_eq!(instance.global_len(), 1);
        let result = instance.global_names();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ["global"]);
    }

    #[test]
    fn test_instance_get() {
        let module_name = "extern_module";

        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();
        assert!(!store.inner.0.is_null());
        assert!(!store.registered);

        // check the length of registered module list in store before instatiation
        assert_eq!(store.module_len(), 0);
        assert!(store.module_names().is_none());

        // create ImportObject instance
        let result = ImportModule::create(module_name);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create(&func_ty, Box::new(real_add), 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0..=u32::MAX);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let memory = {
            let result = MemType::create(10..=20);
            assert!(result.is_ok());
            let mem_ty = result.unwrap();
            let result = Memory::create(&mem_ty);
            assert!(result.is_ok());
            result.unwrap()
        };
        import.add_memory("mem", memory);

        // add globals
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Executor::create(Some(config), None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        let import = ImportObject::Import(import);
        let result = executor.register_import_object(&mut store, &import);
        assert!(result.is_ok());

        let result = store.module(module_name);
        assert!(result.is_ok());
        let instance = result.unwrap();

        // get the exported memory
        let result = instance.get_memory("mem");
        assert!(result.is_ok());
        let memory = result.unwrap();
        let result = memory.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert_eq!(ty.limit(), 10..=20);
    }

    fn create_vm() -> Vm {
        let module_name = "extern_module";

        // create ImportModule instance
        let result = ImportModule::create(module_name);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // add host function
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result = Function::create(&func_ty, Box::new(real_add), 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        import.add_func("add", host_func);

        // add table
        let result = TableType::create(RefType::FuncRef, 0..=u32::MAX);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Table::create(&ty);
        assert!(result.is_ok());
        let table = result.unwrap();
        import.add_table("table", table);

        // add memory
        let result = MemType::create(0..=u32::MAX);
        assert!(result.is_ok());
        let mem_ty = result.unwrap();
        let result = Memory::create(&mem_ty);
        assert!(result.is_ok());
        let memory = result.unwrap();
        import.add_memory("mem", memory);

        // add global
        let result = GlobalType::create(ValType::F32, Mutability::Const);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Global::create(&ty, WasmValue::from_f32(3.5));
        assert!(result.is_ok());
        let global = result.unwrap();
        import.add_global("global", global);

        let result = Vm::create(None, None);
        assert!(result.is_ok());
        let mut vm = result.unwrap();

        let result = vm.register_wasm_from_import(ImportObject::Import(import));
        assert!(result.is_ok());

        vm
    }

    fn real_add(inputs: Vec<WasmValue>) -> Result<Vec<WasmValue>, u8> {
        if inputs.len() != 2 {
            return Err(1);
        }

        let a = if inputs[0].ty() == ValType::I32 {
            inputs[0].to_i32()
        } else {
            return Err(2);
        };

        let b = if inputs[1].ty() == ValType::I32 {
            inputs[1].to_i32()
        } else {
            return Err(3);
        };

        let c = a + b;

        Ok(vec![WasmValue::from_i32(c)])
    }
}
