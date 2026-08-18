import gdb
import os
import sys
import re
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger


class GetBaseAddr(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "get_base_addr"
        self.__num_args = 1
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(GetBaseAddr, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {}".format(self.__command_name, "<binary_path>"))
        print("Eg. {} {}".format(self.__command_name, "/usr/lib/example_lib.so"))
        print("    - command name: {}".format(self.__command_name))
        print("    - number of arguments: {}".format(self.__num_args))
        print("    - ret variable in gdb: {}".format(self.__ret_variable_gdb))
        print("      -> returns 0x0 on failure, otherwise a valid address")
        return

    def __check_arguments(self,
                          args: str) -> bool:
        pattern = r'^"(.+)"$'
        match = re.match(pattern, args)
        if not match:
            self.__messenger.print_message(Severity.ERROR, 
                                           "Please encapsulate your string with double quotes (\"\")\n" 
                                           "If you want to find patterns with \", remember to escape with \\")
            return False

        return True

    def invoke(self,
               args: str,
               from_tty: bool = False) -> None:
        if not self.__check_arguments(args):
            self.__usage()
            return
        
        binary_path = args.split('"')[1]
        mappings = gdb.execute("info proc mappings", to_string=True).split("\n")[4:]
        for mapping in mappings:
            if os.path.abspath(binary_path) in mapping:
                tokens = re.split(r"\s+", mapping.strip())
                self.__messenger.print_message(Severity.INFO, "Found base address of {}: {}".format(binary_path, tokens[0]))
                gdb.execute("set {} = {}".format(self.__ret_variable_gdb, tokens[0]))
                return

        self.__messenger.print_message(Severity.ERROR, "Unable to get base address of {}!\n".format(binary_path))
        gdb.execute("set {} = 0x0".format(self.__ret_variable_gdb))
        return


GetBaseAddr()
