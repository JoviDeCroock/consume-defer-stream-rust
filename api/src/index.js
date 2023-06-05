import { createServer } from 'node:http'
import { createSchema, createYoga } from 'graphql-yoga'
import { useDeferStream } from '@graphql-yoga/plugin-defer-stream'
 
const wait = (time) =>
  new Promise((resolve) => setTimeout(resolve, time))
 
const resolvers = {
  Query: {
    async *alphabet() {
      for (const character of ['a', 'b', 'c', 'd', 'e', 'f', 'g']) {
        yield character
        await wait(1000)
      }
    },
    fastField: async () => {
      await wait(100)
      return 'I am speed'
    },
    slowField: async (_, { waitFor }) => {
      await wait(waitFor)
      return 'I am slow'
    }
  }
}
 
const yoga = createYoga({
  schema: createSchema({
    typeDefs,
    resolvers
  }),
  plugins: [useDeferStream()]
})
 
const server = createServer(yoga)
 
server.listen(4000, () => {
  console.info('Server is running on http://localhost:4000/graphql')
})