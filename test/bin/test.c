#include <stdio.h>

int func_1(void) {
    printf("In Func 1\n");
    int x = 1;
    int y = 2;
    int z = 3;
    int result = 0;

    result = y*z + x*x;
    return result;
}

void func_2(char *some_string) {
    printf("In Func 2\n");
    printf("some_string: %s\n", some_string);
    return;
}

void func_3(char *json_string) {
    printf("In Func 3\n");
    printf("json_string: %s\n", json_string);
    return;
}

int main(void) {
    int choice = 0;
    char *test = "test string";
    char *users = "{\n"
              "    \"user1\": {\n"
              "        \"id\": 1,\n"
              "        \"name\": \"David\"\n"
              "    },\n"
              "    \"user2\": {\n"
              "        \"id\": 2,\n"
              "        \"name\": \"Tom\"\n"
              "    },\n"
              "    \"user3\": {\n"
              "        \"id\": 3,\n"
              "        \"name\": \"Ken\"\n"
              "    }\n"
              "}";
    while(1) {
        printf("Choice: ");
        scanf("%d", &choice);
        switch(choice) {
            case 1:
                func_1();
                break;
            case 2:
                func_2(test);
                break;
            case 3:
                func_3(users);
                break;
            default:
                printf("Invalid choice! Please try again\n");
        }
    }
    return 0;
}
