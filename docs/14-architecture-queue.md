# Architecture: queue

In-memory FIFO. The capture path is `capture -> queue.append`; the
drain path is `queue.pop -> drainer.write`. A `Mutex<()>` around the
queue is enough — the critical section is microseconds.

The queue has no persistence by design. A crash mid-capture loses
the in-flight URL; the next hotkey press re-captures it. This is
the right trade for a personal tool: durability matters more for
the settled vault than for the live edge.
