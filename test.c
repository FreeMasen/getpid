#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/sysctl.h>
#include <pwd.h>

int main(void) {
    int err = 0;
    struct kinfo_proc *proc_list = NULL;
    size_t length = 0;

    static const int name[] = { CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0 };

    // Call sysctl with a NULL buffer to get proper length
    printf("len: %lu", (sizeof(name) / sizeof(*name)));
    err = sysctl((int *)name, (sizeof(name) / sizeof(*name)) - 1, NULL, &length, NULL, 0);
    if (err) goto ERROR;

    // Allocate buffer
    proc_list = malloc(length);
    if (!proc_list) goto ERROR;

    // Get the actual process list
    err = sysctl((int *)name, (sizeof(name) / sizeof(*name)) - 1, proc_list, &length, NULL, 0);
    if (err) goto ERROR;

    int proc_count = length / sizeof(struct kinfo_proc);

    // use getpwuid_r() if you want to be thread-safe

    for (int i = 0; i < proc_count; i++) {
        uid_t uid = proc_list[i].kp_eproc.e_ucred.cr_uid;
        struct passwd *user = getpwuid(uid);
        char *username = user ? user->pw_name : "user name not found";
        char * sess_name = proc_list[i].kp_proc.p_comm;
        // printf("pid=%d, uid=%d, username=%s\n",
        //         proc_list[i].kp_proc.p_pid,
        //         uid,
        //         sess_name);
    }

    free(proc_list);

    return EXIT_SUCCESS;

ERROR:
    perror(NULL);
    free(proc_list);
    return EXIT_FAILURE;
}