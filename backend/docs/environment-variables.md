## Environment Variables

| Name                  | Description                                                           |
| --------------------- | --------------------------------------------------------------------- |
| `MYSQL_ROOT_PASSWORD` | the password that will be set for the MySQL `root` superuser account. |
| `MYSQL_DATABASE`      | the name of a database to be created on image startup. |
| `MYSQL_USER`          | create a new user and to set that user's password. This user will be granted superuser permissions for the database specified by the `MYSQL_DATABASE`. |
| `MYSQL_PASSWORD`      | ( Likewise above ) |

refs
[mysql - Official Image | Docker Hub](https://hub.docker.com/_/mysql)
[MySQL :: MySQL 5.7 Reference Manual :: 4.9 Environment Variables](https://dev.mysql.com/doc/refman/5.7/en/environment-variables.html)
