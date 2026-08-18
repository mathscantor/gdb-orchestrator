import gdb
import os
import sys
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger


class LoadLibrarySymbol(gdb.Command):

    def __init__(self):
        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "load_library_symbol"
        self.__num_args = 2
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(LoadLibrarySymbol, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __check_arguments(self,
                          arg_tokens: List[str]) -> bool:
        if len(arg_tokens) != self.__num_args:
            self.__messenger.print_message(Severity.ERROR, "{}: Expected {} arguments but "
                                                           "got {} arguments instead!".format(self.__command_name,
                                                                                              self.__num_args,
                                                                                              len(arg_tokens)))
            return False
        return True

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {} {}".format(self.__command_name, "<library dir>", "<library name>"))
        print("Eg. {} {} {}".format(self.__command_name, "/usr/lib/x86_64-linux-gnu", "ld-linux-x86-64.so.2"))
        print("    - command name: {}".format(self.__command_name))
        print("    - number of arguments: {}".format(self.__num_args))
        print("    - ret variable in gdb: {}".format(self.__ret_variable_gdb))
        print("      -> returns 1 on failure, 0 on success")
        return

    def invoke(self,
               args: str,
               from_tty: bool = False) -> None:
        arg_tokens = [arg.strip() for arg in args.split()]
        if not self.__check_arguments(arg_tokens):
            self.__usage()
            return

        library_dir = arg_tokens[0]
        library_name = arg_tokens[1]
        if not os.path.isdir(library_dir):
            self.__messenger.print_message(Severity.ERROR, "Unable to find directory: {}".format(library_dir))
            gdb.execute("set {} = 1".format(self.__ret_variable_gdb))
            return
        gdb.execute("set solib-search-path {}".format(library_dir), to_string=True)

        if not os.path.exists(os.path.join(library_dir, library_name)):
            self.__messenger.print_message(Severity.ERROR, "Unable to find {} in {} directory!".format(library_name,
                                                                                                       library_dir))
            gdb.execute("set {} = 1".format(self.__ret_variable_gdb))
            return

        gdb.execute("sharedlibrary " + library_name, to_string=True)
        ret = gdb.execute("info sharedlibrary {}".format(library_name), to_string=True)
        if not ret:
            self.__messenger.print_message(Severity.ERROR, "Unable to load symbols from library: {}".format(library_name))
            gdb.execute("set {} = 1".format(self.__ret_variable_gdb))

        gdb.execute("set {} = 0".format(self.__ret_variable_gdb))
        return


LoadLibrarySymbol()
