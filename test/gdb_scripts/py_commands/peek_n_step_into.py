import gdb
import os
import sys
import re
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger


class PeekNStepInto(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "peek_n_step_into"
        self.__num_args = 1
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(PeekNStepInto, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        self.__file_path_regex = re.compile(r'".*"')

        # Attempted to suppress ugly output from si but failed
        # output = gdb.execute("show logging", to_string=True)
        # self.__current_logging_file = ""
        # for line in output.split("\n"):
        #     logging_option_breakdown = [token.strip() for token in line.split(":")]
        #     if len(logging_option_breakdown) == 2 and logging_option_breakdown[0] == "logging file":
        #         match = self.__file_path_regex.search(logging_option_breakdown[1])
        #         if match:
        #             self.__current_logging_file = match.group(0)
        #             break
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {}".format(self.__command_name, "<number of step-into instructions>"))
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

        self.__messenger.print_message(Severity.INFO, "Peeking {} Step-Into:".format(num_steps))
        for i in range(num_steps):
            # gdb's redirection is bugged lol
            gdb.execute("stepi", to_string=True)
            gdb.execute("x/i $pc")
        gdb.execute("continue")
        gdb.execute("set {} = 0".format(self.__ret_variable_gdb))
        return


PeekNStepInto()
