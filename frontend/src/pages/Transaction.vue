<template>
  <div>
    <div class="container mt-5">
      <div class="row">
        <div class="col-sm-12">
          <h1>Transaction</h1>

          <ul class="list-group mt-5">
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Hash:</strong></div>
                <div class="col-sm-9">
                  <code>{{ hash }}</code>
                </div>
              </div>
            </li>
            <li v-if="location.block_height" class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Block:</strong></div>
                <div class="col-sm-9">
                  <router-link :to="{ name: 'block', params: { height: location.block_height } }">{{ location.block_height }}</router-link>
                </div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Type:</strong></div>
                <div class="col-sm-9">
                  <code>{{ type }}</code>
                </div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Status:</strong></div>
                <div class="col-sm-9">
                  <code>{{ status.type }}</code>
                </div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Protocol version:</strong></div>
                <div class="col-sm-9">{{ transaction.protocol_version }}</div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Service ID:</strong></div>
                <div class="col-sm-9">{{ transaction.service_id }}</div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Transaction ID:</strong></div>
                <div class="col-sm-9">{{ transaction.message_id }}</div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Signature:</strong></div>
                <div class="col-sm-9">
                  <code>{{ transaction.signature }}</code>
                </div>
              </div>
            </li>
            <li class="list-group-item">
              <div class="row">
                <div class="col-sm-3"><strong>Transaction body:</strong></div>
                <div class="col-sm-9">
                  <pre><code>{{ JSON.stringify(transaction.body, null, 2) }}</code></pre>
                </div>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>

    <spinner :visible="isSpinnerVisible"/>
  </div>
</template>

<script>
  import Spinner from '../components/Spinner.vue'

  module.exports = {
    components: {
      Spinner
    },
    props: {
      hash: String
    },
    data() {
      return {
        transaction: {},
        location: {},
        status: {},
        type: ''
      }
    },
    methods: {
      async loadTransaction() {
        try {
          const data = await this.$blockchain.getTransaction(this.hash)
          this.transaction = data.content
          this.location = data.location
          this.status = data.status
          this.type = data.type
          this.isSpinnerVisible = false
        } catch (error) {
          this.isSpinnerVisible = false
          this.$notify('error', error.toString())
        }
      }
    },
    mounted() {
      this.$nextTick(function() {
        this.loadTransaction()
      })
    }
  }
</script>
