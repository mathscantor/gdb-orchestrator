#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <signal.h>
#include <unistd.h>

#define NUM_THREADS 5

volatile sig_atomic_t running = 1;

void func_1() {
  printf("Func 1: Counting up...\n");
  return;
}

void func_2() {
  printf("Func 2: Counting down...\n");
  return;
}

void func_3() {
  printf("Func 3: Printing stars ***\n");
  return;
}

void func_4() {
  printf("Thread 4: Printing dashes ---\n");
  return;
}

void func_5() {
  printf("Thread 5: Printing dots ...\n");
  return;
}

void handle_sigint(int sig) {
    running = 0;
}

void* thread_func_1(void* arg) {
    while (running) {
        func_1();
        sleep(1);
    }
    return NULL;
}

void* thread_func_2(void* arg) {
    while (running) {
        func_2();
        sleep(1);
    }
    return NULL;
}

void* thread_func_3(void* arg) {
    while (running) {
        func_3();
        sleep(1);
    }
    return NULL;
}

void* thread_func_4(void* arg) {
    while (running) {
        func_4();
        sleep(1);
    }
    return NULL;
}

void* thread_func_5(void* arg) {
    while (running) {
        func_5();
        sleep(1);
    }
    return NULL;
}

int main() {
    signal(SIGINT, handle_sigint);
    pthread_t threads[NUM_THREADS];

    pthread_create(&threads[0], NULL, thread_func_1, NULL);
    pthread_create(&threads[1], NULL, thread_func_2, NULL);
    pthread_create(&threads[2], NULL, thread_func_3, NULL);
    pthread_create(&threads[3], NULL, thread_func_4, NULL);
    pthread_create(&threads[4], NULL, thread_func_5, NULL);

    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(threads[i], NULL);
    }

    printf("All threads completed.\n");
    return EXIT_SUCCESS;
}
