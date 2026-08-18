import gdb
import os
import sys
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger


class PeekNStepOver(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "peek_n_step_over"
        self.__num_args = 1
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(PeekNStepOver, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {}".format(self.__command_name, "<number of step-over instructions>"))
        print("Eg. {} {}".format(self.__command_name, "$num_steps"))
        print("Eg. {} {}".format(self.__command_name, "0x16"))
        print("Eg. {} {}".format(self.__command_name, "1000"))
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
            return

        num_steps = 0
        # if the argument passed in is a gdb variable
        if arg_tokens[0].startswith("$"):
            value = str(gdb.parse_and_eval(arg_tokens[0]))
            if value.startswith("0x"):
                num_steps = int(value.split()[0], base=16)
            else:
                num_steps = int(value.split()[0], base=10)

        # if the argument passed in is a hexadecimal address
        elif arg_tokens[0].startswith("0x"):
            num_steps = int(arg_tokens[0], base=16)

        # if the argument passed in is a decimal address
        else:
            num_steps = int(arg_tokens[0], base=10)

        self.__messenger.print_message(Severity.INFO, "Peeking {} Step-Over:".format(num_steps))
        for i in range(num_steps):
            gdb.execute("n")
            ret = gdb.execute("x/i $pc", to_string=True)
            print(ret)
        gdb.execute("continue")
        gdb.execute("set {} = 0".format(self.__ret_variable_gdb))
        return


PeekNStepOver()
