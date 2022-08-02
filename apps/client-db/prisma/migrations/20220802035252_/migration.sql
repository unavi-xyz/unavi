-- CreateTable
CREATE TABLE
  "Internal" ("secret" TEXT NOT NULL);

-- CreateTable
CREATE TABLE
  "Project" (
    "id" SERIAL NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "owner" TEXT NOT NULL,
    CONSTRAINT "Project_pkey" PRIMARY KEY ("id")
  );

-- CreateIndex
CREATE UNIQUE INDEX "Internal_secret_key" ON "Internal" ("secret");