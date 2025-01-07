use super::filesystem_mount_definition::FilesystemMountDefinition;

#[derive(Debug, Clone)]
pub struct MountState {
    pub definition: FilesystemMountDefinition,

    /// Tells if the filesystem is currently mounted.
    pub mounted: bool,
}

impl MountState {
    pub fn new(definition: FilesystemMountDefinition, mounted: bool) -> Self {
        Self {
            definition,
            mounted,
        }
    }
}
