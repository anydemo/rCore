use self::kernelvm::ProviderImpl;
use lazy_static::lazy_static;
use rcore_lkm::manager::ModuleManager;
use spin::Mutex;

pub mod kernelvm;

lazy_static! {
    pub static ref LKM_MANAGER: Mutex<ModuleManager> = {
        let mut kmm = ModuleManager::new(ProviderImpl::default());
        Mutex::new(kmm)
    };
}
