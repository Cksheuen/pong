import { useEffect, useRef } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GlTFLoader.js";
import { useImperativeHandle, forwardRef } from "react";

import { UpdateModelProps } from "../type/func";

const scale = 40;

const ThreeScene = forwardRef((_props: any, ref: any) => {
  const threeContainer = useRef<HTMLDivElement | null>(null);
  const model = useRef<THREE.Group | null>(null);
  const update = useRef<boolean>(false);

  async function init() {
    const scene = new THREE.Scene();

    const camera = new THREE.PerspectiveCamera(
      75,
      threeContainer.current!.clientWidth /
        threeContainer.current!.clientHeight,
      0.1,
      1000
    );

    const renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
    });

    renderer.setClearColor(0x000000, 0);

    const loader = new GLTFLoader();
    loader.load(
      "/models/pong-racket.glb",
      function (gltf) {
        /* const model = gltf.scene;
        model.scale.set(scale, scale, scale);
        model.position.set(0, 0, 0);
        model.rotation.set(-Math.PI / 2, 0, 0); */
        model.current = gltf.scene;
        model.current.scale.set(scale, scale, scale);
        model.current.position.set(0, 0, 0);
        handleRotate(0, 0, 0);
        // model.current.traverse((child) => {

        scene.add(model.current);
        console.log("success load");
        animate();
        update.current = true;
      },
      undefined,
      function (error) {
        console.error("An error happened", error);
      }
    );

    scene.background = new THREE.Color(0xffffff);

    const light = new THREE.AmbientLight(0x404040, 100); // 柔和的白光
    scene.add(light);

    renderer.setSize(
      threeContainer.current!.clientWidth,
      threeContainer.current!.clientHeight
    );
    threeContainer.current!.appendChild(renderer.domElement);

    // const geometry = new THREE.BoxGeometry(1, 1, 1);
    // const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    // const cube = new THREE.Mesh(geometry, material);
    // scene.add(cube);

    camera.position.z = 10;

    function animate() {
      requestAnimationFrame(animate);
      if (update.current) {
        update.current = false;
        if (model.current) {
          renderer.render(scene, camera);
          // console.log("render");
        }
      }
    }
    // animate();
  }

  useEffect(() => {
    if (threeContainer.current!.childElementCount > 0) return;
    init();
  }, []);

  function handleRotate(alpha: number, _beta: number, gamma: number) {
    if (model.current) {
      model.current.rotation.set(
        Math.PI * 0.25, // + beta,
        Math.PI + alpha,
        gamma
      );
    }
  }

  useImperativeHandle(ref, () => (updateModelProps: UpdateModelProps) => {
    const alpha = updateModelProps.heading; //* (Math.PI / 180);
    const beta = updateModelProps.beta * (Math.PI / 180);
    const gamma = updateModelProps.gamma;
    handleRotate(alpha, beta, gamma);
    update.current = true;
  });

  return (
    <div
      ref={threeContainer}
      className="flex-1 relative overflow-hidden w-full h-full"
    ></div>
  );
});

export default ThreeScene;
