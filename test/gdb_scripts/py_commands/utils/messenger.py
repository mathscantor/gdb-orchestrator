from enum import Enum


class Severity(Enum):
    DEBUG = "\033[0;94m[DEBUG]\033[0m "
    INFO = "\033[0;92m[INFO]\033[0m "
    WARNING = "\033[0;93m[WARNING]\033[0m "
    ERROR = "\033[0;91m[ERROR]\033[0m "


class Messenger:

    def __init__(self,
                 verbosity_level: int = 1):
        self.__verbosity_level = verbosity_level
        return

    def print_message(self,
                      sev: Severity,
                      message: str = "") -> None:
        """
        Utility function to print messages of different severity levels

        :param sev: A Severity Enum
        :param message: The string to be printed
        :return: None
        """

        # If verbosity is default, then ignore all the debug messages
        if self.__verbosity_level == 1 and sev == Severity.DEBUG:
            return

        # Prints all messages under the sun
        print(sev.value + message)
        return
