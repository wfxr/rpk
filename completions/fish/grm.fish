# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_grm_global_optspecs
	string join \n q/quiet v/verbose color= config-dir= data-dir= cache-dir= bin-dir= h/help V/version
end

function __fish_grm_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_grm_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_grm_using_subcommand
	set -l cmd (__fish_grm_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c grm -n "__fish_grm_needs_command" -l color -d 'This flag controls when to use colors' -r -f -a "{always\t'Force color output',auto\t'Intelligently guess whether to use color output',never\t'Force disable color output'}"
complete -c grm -n "__fish_grm_needs_command" -l config-dir -d 'The configuration directory' -r -F
complete -c grm -n "__fish_grm_needs_command" -l data-dir -d 'The directory to store package data' -r -F
complete -c grm -n "__fish_grm_needs_command" -l cache-dir -d 'The directory to store downloaded packages' -r -F
complete -c grm -n "__fish_grm_needs_command" -l bin-dir -d 'The directory installed binaries linked to' -r -F
complete -c grm -n "__fish_grm_needs_command" -s q -l quiet -d 'Suppress any informational output'
complete -c grm -n "__fish_grm_needs_command" -s v -l verbose -d 'Use verbose output'
complete -c grm -n "__fish_grm_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c grm -n "__fish_grm_needs_command" -s V -l version -d 'Print version'
complete -c grm -n "__fish_grm_needs_command" -f -a "init" -d 'Initialize a new config file'
complete -c grm -n "__fish_grm_needs_command" -f -a "sync" -d 'install any missing packages, re-generating the lock file'
complete -c grm -n "__fish_grm_needs_command" -f -a "add" -d 'Add a new plugin to the config file'
complete -c grm -n "__fish_grm_needs_command" -f -a "restore" -d 'Restore packages to the state in the lockfile'
complete -c grm -n "__fish_grm_needs_command" -f -a "update" -d 'Update packages and re-generate the lock file'
complete -c grm -n "__fish_grm_needs_command" -f -a "search" -d 'Search for packages on GitHub'
complete -c grm -n "__fish_grm_needs_command" -f -a "completions" -d 'Generate completions for the given shell'
complete -c grm -n "__fish_grm_needs_command" -f -a "version" -d 'Prints detailed version information'
complete -c grm -n "__fish_grm_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c grm -n "__fish_grm_using_subcommand init" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand sync" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand add" -l name -d 'A unique name for the package' -r
complete -c grm -n "__fish_grm_using_subcommand add" -l version -d 'The version of the package' -r
complete -c grm -n "__fish_grm_using_subcommand add" -l desc -d 'A description of the package' -r
complete -c grm -n "__fish_grm_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c grm -n "__fish_grm_using_subcommand restore" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand update" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand search" -l top -d 'The number of results to display' -r
complete -c grm -n "__fish_grm_using_subcommand search" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand completions" -s d -l dir -d 'The directory to write the completions to' -r -F
complete -c grm -n "__fish_grm_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c grm -n "__fish_grm_using_subcommand version" -s h -l help -d 'Print help'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "init" -d 'Initialize a new config file'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "sync" -d 'install any missing packages, re-generating the lock file'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "add" -d 'Add a new plugin to the config file'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "restore" -d 'Restore packages to the state in the lockfile'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "update" -d 'Update packages and re-generate the lock file'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "search" -d 'Search for packages on GitHub'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "completions" -d 'Generate completions for the given shell'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "version" -d 'Prints detailed version information'
complete -c grm -n "__fish_grm_using_subcommand help; and not __fish_seen_subcommand_from init sync add restore update search completions version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
