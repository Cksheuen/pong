import { useRef } from "react";
import "./App.css";
import OrientationGetter from "./components/OrientationGetter.tsx";
import ThreeScene from "./components/ThreeScene.tsx";

import { UpdateModelFunc, UpdateModelProps } from "./type/func.ts";

function App() {
  const updateModel = useRef<UpdateModelFunc | null>(null);
  const startUpdate = (updateModelProps: UpdateModelProps) => {
    if (updateModel.current) {
      updateModel.current(updateModelProps);
    }
  };
  return (
    <div className="flex items-center justify-center flex-col h-full overflow-hidden pt-10  pb-10 w-full box-border">
      <div>
        <h1>Pong Controller</h1>
      </div>
      <ThreeScene ref={updateModel} />
      <OrientationGetter updateModel={startUpdate} />
      {/* <div className="absolute top-[50%] h-[50%] w-full z--1 bg-red"></div> */}
    </div>
  );
}

export default App;
