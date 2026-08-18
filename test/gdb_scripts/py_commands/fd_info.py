import gdb
import os
import stat
import sys
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger
from enum import Enum


class FDType(Enum):

    MISSING = -1
    UNKNOWN = 0
    UNIX_SOCKET = 1
    REGULAR_FILE = 2
    DIRECTORY = 3
    SYMLINK = 4
    FIFO_PIPE = 5
    CHAR_DEVICE = 6
    BLOCK_DEVICE = 7

    @classmethod
    def _missing_(cls, value):
        return cls.UNKNOWN


class FDInfo(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "fd_info"
        self.__num_args = 1
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger()
        super(FDInfo, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {}".format(self.__command_name, "<fd number>"))
        print("Eg. {} {}".format(self.__command_name, "$fd"))
        print("    - command name: {}".format(self.__command_name))
        print("    - number of arguments: {}".format(self.__num_args))
        print("    - ret variable in gdb: {}".format(self.__ret_variable_gdb))
        # See FDType class above
        print("      -> returns the fd type. (-1 on failure)")
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

    def __get_fd_type(self,
                      pid: int,
                      fd: int) -> FDType:
        
        fd_path = "/proc/{}/fd/{}".format(pid, fd)
        try:
            file_name = os.readlink(fd_path)
            self.__messenger.print_message(Severity.INFO, "FD Readlink: {}".format(file_name))
            fd_info = os.stat(fd_path)
            if stat.S_ISSOCK(fd_info.st_mode):
                return FDType.UNIX_SOCKET
            elif stat.S_ISREG(fd_info.st_mode):
                return FDType.REGULAR_FILE
            elif stat.S_ISDIR(fd_info.st_mode):
                return FDType.DIRECTORY
            elif stat.S_ISLNK(fd_info.st_mode):
                return FDType.SYMLINK
            elif stat.S_ISFIFO(fd_info.st_mode):
                return FDType.FIFO_PIPE
            elif stat.S_ISCHR(fd_info.st_mode):
                return FDType.CHAR_DEVICE
            elif stat.S_ISBLK(fd_info.st_mode):
                return FDType.BLOCK_DEVICE

        except FileNotFoundError:
            return FDType.MISSING

        return FDType.UNKNOWN

    def invoke(self,
               args: str,
               from_tty: bool = False) -> None:
        arg_tokens = [arg.strip() for arg in args.split()]
        if not self.__check_arguments(arg_tokens):
            self.__usage()
            gdb.execute("set {} = 1".format(self.__ret_variable_gdb))
            return

        fd_number = 0x0
        # if the argument passed in is a gdb variable
        if arg_tokens[0].startswith("$"):
            value = str(gdb.parse_and_eval(arg_tokens[0]))
            if value.startswith("0x"):
                fd_number = int(value.split()[0], base=16)
            else:
                fd_number = int(value.split()[0], base=10)

        # if the argument passed in is hexadecimal format
        elif arg_tokens[0].startswith("0x"):
            fd_number = int(arg_tokens[0], base=16)

        # if the argument passed in is decimal format
        else:
            fd_number = int(arg_tokens[0], base=10)

        attached_pid = gdb.selected_inferior().pid
        fd_type = self.__get_fd_type(attached_pid, fd_number)
        if fd_type == FDType.MISSING:
            self.__messenger.print_message(Severity.ERROR, "FD does not exist!")
        else:
            self.__messenger.print_message(Severity.INFO, "FD Type: {}".format(fd_type.name))

        gdb.execute("set {} = {}".format(self.__ret_variable_gdb, fd_type.value))
        return


FDInfo()
