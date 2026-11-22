# The client of the CARLA server: its Python API (x86-64 only) and an MQTT client.
FROM python:3.10-slim
RUN pip install --no-cache-dir carla==0.9.15 "paho-mqtt>=2,<3"
WORKDIR /client
COPY bridge.py .
ENTRYPOINT ["python", "bridge.py"]
