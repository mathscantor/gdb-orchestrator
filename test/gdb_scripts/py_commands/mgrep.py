import gdb
import os
import sys
import re
import json
import ast
from typing import *
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from utils.messenger import Severity, Messenger
from utils.memory_mapping_handler import MemoryMappingHandler


class Mgrep(gdb.Command):

    def __init__(self):

        """
        Please follow this template and change the following variables:
            1. self.__command_name
            2. self.__num_args
            3. self.__ret_variable_gdb
        """
        self.__command_name = "mgrep"
        self.__num_args = 1
        self.__ret_variable_gdb = "${}_ret".format(self.__command_name)

        self.__messenger = Messenger(verbosity_level=2)
        self.__mm_handler = MemoryMappingHandler()
        self.__pattern_found_regex = re.compile(r'^(\d+) pattern|patterns found\.$')
        super(Mgrep, self).__init__(self.__command_name, gdb.COMMAND_USER)
        self.__usage()
        return

    def __usage(self) -> None:
        self.__messenger.print_message(Severity.INFO, "Example usage of {}:".format(self.__command_name))
        print("Template: {} {}".format(self.__command_name, "\"<expression>\""))
        print("Eg. {} {}".format(self.__command_name, "\"string_to_find\""))
        print("    - command name: {}".format(self.__command_name))
        print("    - number of arguments: {}".format(self.__num_args))
        print("    - ret variable in gdb: {}".format(self.__ret_variable_gdb))
        print(r"      -> returns the number of matches on success(>= 0) and -1 on failure.")
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

    def __find_pattern(self, 
                       memory_sections: Dict[str, Dict[str, str]],
                       expr: str) -> List[str]:
        
        match_list = list()
        bytes_expr = ', '.join([f"0x{ord(char):02x}" for char in expr])
        for section_name in memory_sections.keys():
            results = gdb.execute("find /b {}, {}, {}".format(memory_sections[section_name]["start_addr"],
                                                              memory_sections[section_name]["end_addr"],
                                                              bytes_expr), to_string=True)
            results_tokens = results.split("\n")
            match = self.__pattern_found_regex.search(results_tokens[-2])
            if match:
                num_addresses = int(match.group(1))
                for addr in results_tokens[0: num_addresses]:
                    match_list.append(addr.split()[0])

        return match_list
    
    def __get_binary_path_from_addr(self,
                                    addr: str,
                                    memory_sections) -> str:
        
        for section, addr_range in memory_sections.items():
            start_addr = int(addr_range["start_addr"], 16)
            end_addr = int(addr_range["end_addr"], 16)
            if start_addr <= int(addr, 16) <= end_addr:
                return section

        return ""
    
    def __pretty_print_pattern_match(self,
                                     memory_sections: Dict[str, Dict[str, str]],
                                     match_list: List[str]) -> None:
        
        if len(match_list) == 0:
            self.__messenger.print_message(Severity.INFO, "Found 0 matches!")
            return

        self.__messenger.print_message(Severity.INFO, "Found {} matches at:".format(len(match_list)))
        for addr in match_list:
            binary_path = self.__get_binary_path_from_addr(addr, memory_sections)
            offset_from_base = hex(int(addr, 16) - int(memory_sections[binary_path]["start_addr"], 16))
            try:
                addr_and_string = ' '.join(gdb.execute("x/s {}".format(addr), to_string=True).strip("\n").split())
            except gdb.error:
                print("\t[-] Encountered error at {}".format(addr))
                continue

            print("\t[+] {} + {} -> {}".format(binary_path,
                                               offset_from_base,
                                               addr_and_string))

        return

    def invoke(self,
               args: str,
               from_tty: bool = False) -> None:

        gdb.execute("set {} = -1".format(self.__ret_variable_gdb))
        if not self.__check_arguments(args):
            self.__usage()
            return

        memory_sections = self.__mm_handler.get_memory_sections()
        expr = args.split('"')[1]
        match_list = self.__find_pattern(memory_sections, expr)
        self.__pretty_print_pattern_match(memory_sections, match_list)
        gdb.execute("set {} = {}".format(self.__ret_variable_gdb, len(match_list)))
        return


Mgrep()
