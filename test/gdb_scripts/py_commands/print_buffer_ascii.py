import gdb
import os
import sys
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger


class PrintBufferAscii(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "print_buffer_ascii"
        self.__num_args = 2
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(PrintBufferAscii, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {} {}".format(self.__command_name, "<Buffer Address>", "<Buffer Length>"))
        print("Eg. {} {} {}".format(self.__command_name, "$buf", 52))
        print("    - command name: {}".format(self.__command_name))
        print("    - number of arguments: {}".format(self.__num_args))
        print("    - ret variable in gdb: {}".format(self.__ret_variable_gdb))
        print("      -> returns 1 on failure, otherwise 0 on success")
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

    def invoke(self,
               args: str,
               from_tty: bool = False) -> None:
        arg_tokens = [arg.strip() for arg in args.split()]
        if not self.__check_arguments(arg_tokens):
            self.__usage()
            gdb.execute("set {} = 1".format(self.__ret_variable_gdb))
            return

        buffer = 0x0
        # if the argument passed in is a gdb variable
        if arg_tokens[0].startswith("$"):
            value = str(gdb.parse_and_eval(arg_tokens[0]))
            if value.startswith("0x"):
                buffer = int(value.split()[0], base=16)
            else:
                buffer = int(value.split()[0], base=10)

        # if the argument passed in is hexadecimal format
        elif arg_tokens[0].startswith("0x"):
            buffer = int(arg_tokens[0], base=16)

        # if the argument passed in is decimal format
        else:
            buffer = int(arg_tokens[0], base=10)

        self.__messenger.print_message(Severity.DEBUG, "Buffer Address: {}".format(buffer))

        length = 0x0
        # if the argument passed in is a gdb variable
        if arg_tokens[1].startswith("$"):
            value = str(gdb.parse_and_eval(arg_tokens[1]))
            if value.startswith("0x"):
                length = int(value.split()[0], base=16)
            else:
                length = int(value.split()[0], base=10)

        # if the argument passed in is hexadecimal format
        elif arg_tokens[1].startswith("0x"):
            length = int(arg_tokens[1], base=16)

        # if the argument passed in is decimal format
        else:
            length = int(arg_tokens[1], base=10)

        self.__messenger.print_message(Severity.DEBUG, "Buffer Length: {}".format(length))

        ascii_string = ""
        for offset in range(length):
            byte = bytes(gdb.inferiors()[0].read_memory(buffer + offset, 1)[0])
            # check if it's a printable ASCII character
            if 32 <= int.from_bytes(byte, "little") <= 126:
                ascii_string += byte.decode('utf-8')
            else:
                ascii_string += "."
            
        self.__messenger.print_message(Severity.INFO, "Buffer (Only Printable ASCII): {}".format(ascii_string))
        gdb.execute("set {} = 0".format(self.__ret_variable_gdb))
        return


PrintBufferAscii()
