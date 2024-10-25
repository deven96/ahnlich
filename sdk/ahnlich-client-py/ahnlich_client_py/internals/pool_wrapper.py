import contextlib
import errno
import platform
import socket
from ipaddress import IPv4Address, IPv6Address
from typing import Generator, Tuple

from generic_connection_pool.contrib.socket import TcpSocketConnectionManager

WINDOWS = "windows"


@contextlib.contextmanager
def socket_nonblocking(sock: socket.socket) -> Generator[None, None, None]:
    orig_timeout = sock.gettimeout()
    # This fixes the `[Win 100035]: A non-blocking socket operation could not be completed immediately` error.
    # Essentially this tells windows this is a blocking socket, thereby allowing check_aliveness exec without errors.
    if platform.system().lower() == WINDOWS:
        sock.settimeout(1e-9)
    else:
        sock.settimeout(0)
    try:
        yield
    finally:
        sock.settimeout(orig_timeout)


class AhnlichTcpSocketConnectionManager(TcpSocketConnectionManager):
    """Subclasses `TcpSocketConnectionManager` to fix the check_aliveness issue on windows"""

    def check_aliveness(
        self,
        endpoint: Tuple[IPv4Address | IPv6Address, int],
        conn: socket.socket,
        timeout: float | None = None,
    ) -> bool:
        try:
            with socket_nonblocking(conn):
                if conn.recv(1, socket.MSG_PEEK) == b"":
                    return False
        except BlockingIOError as exc:
            if exc.errno != errno.EAGAIN:
                raise
        except OSError:
            return False
        return True
