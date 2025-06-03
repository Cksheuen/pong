import { useState, useEffect, useRef } from "react";
import * as THREE from "three";

import { UpdateModelFunc } from "../type/func";

interface OrientationGetterProps {
  updateModel: UpdateModelFunc;
}

interface OrientationPosition {
  alpha: number;
  beta: number;
  gamma: number;
}

export default function OrientationGetter({
  updateModel,
}: OrientationGetterProps) {
  const [gamma, setGamma] = useState(0);
  const [beta, setBeta] = useState(0);
  const [alpha, setAlpha] = useState(0);
  const ws = useRef<WebSocket | null>(null);
  const status = useRef<boolean>(false);
  let orientationPermission: boolean = false;
  const wsStartStatus = useRef<boolean>(false);

  const refGamma = useRef(0);

  // const [originHeading, setOriginHeading] = useState<number | null>(null); // useRef<number | null>(null);
  const [heading, setHeading] = useState(0);
  const originHeading = useRef<number | null>(null);

  function handleOrientation(event: any) {
    const newAlpha = event.alpha;
    const newBeta = event.beta;
    const newGamma = event.gamma;
    // 更新状态
    setAlpha(newAlpha.toFixed(0));
    setBeta(newBeta.toFixed(0));
    setGamma(newGamma.toFixed(0));
    refGamma.current = parseInt(newGamma.toFixed(0));

    const alphaRad = (newAlpha * Math.PI) / 180;
    const betaRad = (newBeta * Math.PI) / 180;
    const gammaRad = (newGamma * Math.PI) / 180;

    // 3. 先把设备欧拉角转成四元数（YXZ 顺序）
    const euler = new THREE.Euler(betaRad, alphaRad, -gammaRad, "YXZ");
    // “YXZ” 顺序：先绕 Y（Yaw），再绕 X（Pitch），最后绕 Z（Roll）
    // 这样 euler.y 就恰好是「绕世界 Y 轴」的航向角
    const quat = new THREE.Quaternion().setFromEuler(euler);
    const yawEuler = new THREE.Euler(0, 0, 0, "YXZ");
    yawEuler.setFromQuaternion(quat);
    let heading = THREE.MathUtils.radToDeg(yawEuler.y);
    if (heading < 0) heading += 360;
    // console.log(Math.round(heading));
    setHeading(Math.round(heading));

    if (originHeading.current) {
      // const head = Math.round(heading);
      let head = Math.abs(heading - originHeading.current);
      head = Math.min(head, 360 - head) * (head > 180 ? 1 : -1);
      head *= Math.PI / 180;

      const alpha = parseInt(newAlpha.toFixed(0)) * (Math.PI / 180);
      const beta = parseInt(newBeta.toFixed(0));
      const gamma = parseInt(newGamma.toFixed(0)) * (Math.PI / 180);
      // gamma = Math.abs(gamma - originPosition.current.gamma);
      // gamma = Math.min(gamma, 360 - gamma) * (gamma > 180 ? 1 : -1);
      // gamma *= Math.PI / 180;

      if (status.current && ws.current)
        ws.current.send(
          `rotation:${
            head //(Math.min(360 - head, head) / 180) * (180 > head ? 1 : -1) * Math.PI
          },${alpha},0,${gamma}`
        );
      // console.log("updateModel", parseInt(newAlpha.toFixed(0)));

      updateModel({
        alpha,
        beta,
        gamma,
        heading: head,
      });
    }
  }

  function getOrientationPermission() {
    if (
      typeof (DeviceOrientationEvent as any).requestPermission === "function"
    ) {
      (DeviceOrientationEvent as any)
        .requestPermission()
        .then((permissionState: PermissionState) => {
          if (permissionState === "granted") {
            // handle data
            window.addEventListener("deviceorientation", handleOrientation);
            orientationPermission = true;
          } else {
            // handle denied
          }
        })
        .catch((err: Error) => {
          console.log(err);
        });
    } else {
      // han
      console.log(typeof DeviceOrientationEvent);
      // setStatusLogs("该浏览器不支持请求权限，直接使用");
    }
  }

  function getPermission() {
    getOrientationPermission();
    getMotionPermission();
  }
  useEffect(() => {
    try {
      if (!orientationPermission) getPermission();
      if (!wsStartStatus.current) {
        wsStartStatus.current = true;
        ws.current = new WebSocket("wss://192.168.1.105:8080");
        // ws = new WebSocket("wss://dev.local:8080");
        ws.current.onopen = () => {
          if (ws.current?.OPEN) {
            ws.current.send("hello");
            status.current = true;
          }
        };

        ws.current.onmessage = (e) => console.log("收到:", e.data);
        ws.current.onerror = (e) => {
          console.log("error:", e);
          // setWsStatusLogs(JSON.stringify(e));
        };
      }
    } catch (error) {
      console.log(error);
    }
  }, []);

  function handleClick() {
    console.log("click");

    try {
      if (ws.current && status.current) {
        console.log("send");

        ws.current.send(
          JSON.stringify({
            alpha,
            beta,
            gamma,
          })
        );
      }
    } catch (err) {
      alert(err);
    }
  }

  function getMotionPermission() {
    if (
      typeof DeviceMotionEvent !== "undefined" &&
      typeof (DeviceMotionEvent as any).requestPermission === "function"
    ) {
      // iOS 13+
      (DeviceMotionEvent as any)
        .requestPermission()
        .then((resp: PermissionState) => {
          if (resp === "granted") {
            window.addEventListener("devicemotion", handleMotion);
          }
        })
        .catch(console.error);
    } else {
      window.addEventListener("devicemotion", handleMotion);
    }
  }

  // const [acc, setAcc] = useState();
  // const [accG, setAccG] = useState();
  // const [rot, setRot] = useState();
  const vX = useRef(0);
  const vY = useRef(0);
  const vZ = useRef(0);

  const [showDelta, setShowDelta] = useState("");
  // const preTimeStamp = useRef<number | null>(null);

  function handleMotion(event: DeviceMotionEvent) {
    const acc = event.acceleration; // 不含重力
    const accG = event.accelerationIncludingGravity; // 含重力
    // const rot = event.rotationRate;
    const interval = event.interval;

    // setAcc(acc);
    // setAccG(accG);
    // setRot(rot);
    /* let interval = 0;
    const newTimeStamp = new Date().getTime();
    if (preTimeStamp.current)
      interval = (newTimeStamp - preTimeStamp.current) / 1000;
    preTimeStamp.current = newTimeStamp; */

    const newVX = accG!.x! * interval + vX.current;
    const newVY = accG!.y! * interval + vY.current;
    const newVZ = accG!.z! * interval + vZ.current;

    const dx = ((newVX + vX.current) / 2) * interval;
    const dy = ((newVY + vY.current) / 2) * interval;
    const dz = ((newVZ + vZ.current) / 2) * interval;

    vX.current = newVX;
    vY.current = newVY;
    vZ.current = newVZ;

    // setShowDelta(status.current.toString());
    const gamma = refGamma.current;

    if (status.current && ws.current) {
      // if (gamma > 0 && gamma < 90) {
      const top = accG!.z! < 0 ? -1 : 1;
      const gammaInRadians = (gamma / 180) * Math.PI;
      let deltaX =
        dz * Math.sin(gammaInRadians) + dx * Math.cos(gammaInRadians);
      deltaX *= -top;
      ws.current.send(`position:${dx.toString()},${accG!.y!.toString()},0`);
      setShowDelta(accG!.y!.toFixed(3).toString());
      // }
    }
  }

  const originPosition = useRef<OrientationPosition>({
    alpha: 0,
    beta: 0,
    gamma: 0,
  });

  function setOriginPosition() {
    originPosition.current = {
      alpha: alpha,
      beta: beta,
      gamma: gamma,
    };
  }

  useEffect(() => {
    if (!originHeading.current && heading) {
      originHeading.current = heading;
      // setOriginHeading(heading);
    }
  }, [heading]);

  return (
    <div className="flex flex-col items-center justify-center">
      <div>showDelta:{showDelta}</div>
      <div>{alpha}</div>
      <button onClick={getPermission}>click to get permission</button>
      <button onClick={handleClick}>click to send message</button>
      <button onClick={setOriginPosition}>set origin position</button>
    </div>
  );
}
