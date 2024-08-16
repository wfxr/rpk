# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_rpk_global_optspecs
	string join \n q/quiet v/verbose color= config-dir= data-dir= cache-dir= bin-dir= h/help V/version
end

function __fish_rpk_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_rpk_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_rpk_using_subcommand
	set -l cmd (__fish_rpk_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c rpk -n "__fish_rpk_needs_command" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_needs_command" -l config-dir -d 'The configuration directory' -r -F
complete -c rpk -n "__fish_rpk_needs_command" -l data-dir -d 'The directory to store package data' -r -F
complete -c rpk -n "__fish_rpk_needs_command" -l cache-dir -d 'The directory to store downloaded packages' -r -F
complete -c rpk -n "__fish_rpk_needs_command" -l bin-dir -d 'The directory installed binaries linked to' -r -F
complete -c rpk -n "__fish_rpk_needs_command" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_needs_command" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_needs_command" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_needs_command" -s V -l version -d 'Print version'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "init" -d 'Initialize a configuration file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "list" -d 'List all installed packages'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "l" -d 'List all installed packages'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "ls" -d 'List all installed packages'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "sync" -d 'Install any missing packages, re-generating the lock file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "s" -d 'Install any missing packages, re-generating the lock file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "add" -d 'Add a new plugin to the config file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "a" -d 'Add a new plugin to the config file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "restore" -d 'Restore packages to the state in the lockfile'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "r" -d 'Restore packages to the state in the lockfile'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "update" -d 'Update packages and re-generate the lock file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "u" -d 'Update packages and re-generate the lock file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "find" -d 'Find packages matching the given query'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "f" -d 'Find packages matching the given query'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "fd" -d 'Find packages matching the given query'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "cleanup" -d 'Remove packages which are not listed in the lock file'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "env" -d 'Prints the environment variables for rpk'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "completions" -d 'Generate completions for the given shell'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "version" -d 'Prints detailed version information'
complete -c rpk -n "__fish_rpk_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c rpk -n "__fish_rpk_using_subcommand init" -s f -l from -d 'The config file URL to initialize from' -r
complete -c rpk -n "__fish_rpk_using_subcommand init" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand init" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand init" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand init" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand list" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand list" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand list" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand list" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand l" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand l" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand l" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand l" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand ls" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand ls" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand ls" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand ls" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand sync" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand sync" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand sync" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand sync" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand s" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand s" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand s" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand s" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand add" -l name -d 'A unique name for the package. Defaults to the repo name' -r
complete -c rpk -n "__fish_rpk_using_subcommand add" -l version -d 'The version of the package' -r
complete -c rpk -n "__fish_rpk_using_subcommand add" -l desc -d 'A description of the package' -r
complete -c rpk -n "__fish_rpk_using_subcommand add" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand add" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand add" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c rpk -n "__fish_rpk_using_subcommand a" -l name -d 'A unique name for the package. Defaults to the repo name' -r
complete -c rpk -n "__fish_rpk_using_subcommand a" -l version -d 'The version of the package' -r
complete -c rpk -n "__fish_rpk_using_subcommand a" -l desc -d 'A description of the package' -r
complete -c rpk -n "__fish_rpk_using_subcommand a" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand a" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand a" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand a" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c rpk -n "__fish_rpk_using_subcommand restore" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand restore" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand restore" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand restore" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand r" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand r" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand r" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand r" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand update" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand update" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand update" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand update" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand u" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand u" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand u" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand u" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand find" -l top -d 'The number of results to display' -r
complete -c rpk -n "__fish_rpk_using_subcommand find" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand find" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand find" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand find" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand f" -l top -d 'The number of results to display' -r
complete -c rpk -n "__fish_rpk_using_subcommand f" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand f" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand f" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand f" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand fd" -l top -d 'The number of results to display' -r
complete -c rpk -n "__fish_rpk_using_subcommand fd" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand fd" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand fd" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand fd" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand cleanup" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand cleanup" -l cache -d 'Remove all cached data as well'
complete -c rpk -n "__fish_rpk_using_subcommand cleanup" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand cleanup" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand cleanup" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand env" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand env" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand env" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand env" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand completions" -s d -l dir -d 'The directory to write the completions to' -r -F
complete -c rpk -n "__fish_rpk_using_subcommand completions" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand completions" -s l -l list -d 'List all available shells'
complete -c rpk -n "__fish_rpk_using_subcommand completions" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand completions" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c rpk -n "__fish_rpk_using_subcommand version" -l color -d 'This flag controls when to use colors' -r -f -a "{auto\t'',always\t'',never\t''}"
complete -c rpk -n "__fish_rpk_using_subcommand version" -s q -l quiet -d 'Suppress any informational output'
complete -c rpk -n "__fish_rpk_using_subcommand version" -s v -l verbose -d 'Use verbose output'
complete -c rpk -n "__fish_rpk_using_subcommand version" -s h -l help -d 'Print help'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "init" -d 'Initialize a configuration file'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "list" -d 'List all installed packages'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "sync" -d 'Install any missing packages, re-generating the lock file'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "add" -d 'Add a new plugin to the config file'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "restore" -d 'Restore packages to the state in the lockfile'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "update" -d 'Update packages and re-generate the lock file'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "find" -d 'Find packages matching the given query'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "cleanup" -d 'Remove packages which are not listed in the lock file'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "env" -d 'Prints the environment variables for rpk'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "completions" -d 'Generate completions for the given shell'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "version" -d 'Prints detailed version information'
complete -c rpk -n "__fish_rpk_using_subcommand help; and not __fish_seen_subcommand_from init list sync add restore update find cleanup env completions version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
