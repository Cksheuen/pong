interface UpdateModelProps {
    heading: number;
    alpha: number;
    beta: number;
    gamma: number;
}

type UpdateModelFunc = (updateModelProps: UpdateModelProps) => void;

export type {
    UpdateModelFunc,
    UpdateModelProps,
}