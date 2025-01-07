# Define all known sftpman commands
set --local commands create update help ls mount mount_all umount umount_all rm preflight_check

# Main command completions
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "create" --no-files --description "Creates a new filesystem mount definition"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "update" --no-files --description "Updates an existing filesystem mount definition"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "help" --no-files --description "Show help"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "ls" --no-files --description "List SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "mount" --no-files --description "Mount SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "mount_all" --no-files --description "Mount all SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "umount" --no-files --description "Unmount SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "umount_all" --no-files --description "Unmount all SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "rm" --no-files --description "Remove SFTP systems"
complete --command sftpman --condition "not __fish_seen_subcommand_from $commands" --arguments "preflight_check" --no-files --description "Check if all system requirements are satisfied"

# ls subcommand completions
complete --command sftpman --condition "__fish_seen_subcommand_from ls" --arguments "available" --no-files --description "List all available systems"
complete --command sftpman --condition "__fish_seen_subcommand_from ls" --arguments "mounted" --no-files --description "List mounted systems"
complete --command sftpman --condition "__fish_seen_subcommand_from ls" --arguments "unmounted" --no-files --description "List unmounted systems"

# mount subcommand completions - suggest unmounted systems
complete --command sftpman --condition "__fish_seen_subcommand_from mount" --arguments "(sftpman ls unmounted)" --no-files

# umount subcommand completions - suggest mounted systems
complete --command sftpman --condition "__fish_seen_subcommand_from umount" --arguments "(sftpman ls mounted)" --no-files

# rm subcommand completions - suggest available systems
complete --command sftpman --condition "__fish_seen_subcommand_from rm" --arguments "(sftpman ls available)" --no-files

# mount_all/umount_all subcommand completions - nothing to suggest
complete --command sftpman --condition "__fish_seen_subcommand_from mount_all umount_all" --no-files

complete --command sftpman --condition "__fish_seen_subcommand_from create update" --no-files \
    --arguments "--" \
    --description "Available options (use --option=value format)"

complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option id --description "Unique identifier" --arguments "(sftpman ls available)" --no-files --require-parameter
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option host --description "Hostname or IP address" --arguments "(__fish_complete_hostnames)" --no-files --require-parameter
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option port --description "SSH port number" --arguments "22" --no-files --require-parameter
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option user --description "Remote username to authenticate with" --arguments "(__fish_complete_users)" --no-files --require-parameter
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option auth_type --description "Authentication method" --arguments "publickey authentication-agent password keyboard-interactive hostbased gssapi-with-mic" --require-parameter
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option ssh_key --description "SSH private key path" -r
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option mount_opt --description "Mount options" --arguments "(sshfs --help 2>&1 | grep '\-o' | cut --description '-' --no-files 2 | cut --description ' ' --no-files 2 | grep -vE '^\$')"
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option remote_path --description "Remote path to mount (e.g. /storage)" --arguments "(__fish_complete_directories)"
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option mount_path --description "Local path to mount to (defaults to /mnt/sshfs/{id})" --arguments "(__fish_complete_directories)"
complete --command sftpman --condition "__fish_seen_subcommand_from create update" --long-option cmd_before_mount --description "Command to run before mounting"
